//! MP4 → MP4 トランスコードパイプライン (H.264 / H.265 / AV1 / AAC / Opus)
//!
//! 1. 入力 MP4 をメモリに読み込み `Mp4FileDemuxer` でトラック情報・サンプルを抽出
//! 2. 映像トラックをデコード → I420 → エンコード
//!    - デコーダ: VideoToolbox (H.264/H.265), dav1d (AV1)
//!    - エンコーダ: VideoToolbox (H.264/H.265), SVT-AV1 (AV1)
//! 3. 音声トラックをデコード → PCM → エンコード
//!    - デコーダ: AudioToolbox (AAC/Opus), shiguredo_opus (Opus)
//!    - エンコーダ: AudioToolbox (AAC), shiguredo_opus (Opus)
//! 4. `Mp4FileMuxer` で映像・音声サンプルを時系列順に書き込み

mod audio;
mod mux;
mod params;
mod video;

use std::num::NonZeroU32;
use std::path::Path;

use shiguredo_mp4::TrackKind;
use shiguredo_mp4::boxes::SampleEntry;
use shiguredo_mp4::demux::{Input, Mp4FileDemuxer, TrackInfo};

use crate::codec::EncodeRecipe;
use crate::encode::JobProgress;
use crate::error::{Error, Result};

use self::video::{
    encode_video,
};
use self::audio::encode_audio;
use self::mux::write_mp4;
use self::params::{
    first_sample_entry, resolution_of, audio_params_of, copy_stride, average_duration,
    average_bitrate_kbps, extract_h265_params, DEFAULT_VIDEO_TIMESCALE,
};

/// デコード前の生サンプル（圧縮データ＋メタデータ）
struct RawSample {
    data: Vec<u8>,
    timestamp: u64,
    duration: u32,
    keyframe: bool,
    composition_time_offset: Option<i64>,
    sample_entry: Option<SampleEntry>,
}

/// エンコード後の映像サンプル
struct EncodedVideoSample {
    data: Vec<u8>,
    keyframe: bool,
    timestamp: u64,
    duration: u32,
    composition_time_offset: Option<i64>,
}

/// エンコード後の音声サンプル
struct EncodedAudioSample {
    data: Vec<u8>,
    timestamp: u64,
    duration: u32,
    composition_time_offset: Option<i64>,
}

/// 出力トラック種別 (mux 時の時系列マージ用)
enum OutputKind {
    Video(EncodedVideoSample),
    Audio(EncodedAudioSample),
}

#[cfg(target_os = "macos")]
struct VideoOutput {
    samples: Vec<EncodedVideoSample>,
    sample_entry: SampleEntry,
}

#[cfg(target_os = "macos")]
struct AudioOutput {
    samples: Vec<EncodedAudioSample>,
    sample_entry: SampleEntry,
    /// 出力音声のサンプルレート (Hz)
    ///
    /// 音声サンプルの timestamp はサンプル単位の累積値なので、
    /// mux 時の timescale には入力トラックの timescale ではなく
    /// このサンプルレートを使う必要がある。
    sample_rate: u32,
}
pub fn transcode(
    input: &Path,
    output: &Path,
    recipe: EncodeRecipe,
    overwrite: bool,
    progress: &JobProgress,
) -> Result<()> {
    let input_data = std::fs::read(input)?;
    tracing::info!(file = %input.display(), size = input_data.len(), "encode started");

    let mut demuxer = Mp4FileDemuxer::new();
    demuxer.handle_input(Input {
        position: 0,
        data: &input_data,
    });
    let tracks: Vec<TrackInfo> = demuxer.tracks()?.to_vec();

    let video_idx = tracks.iter().position(|t| t.kind == TrackKind::Video);
    let audio_idx = tracks.iter().position(|t| t.kind == TrackKind::Audio);

    // 映像/音声トラックが一つもなければ出力対象がないため早期 return する
    // (write_mp4 に None, None を渡すと破損 MP4 が生成されるため)
    if video_idx.is_none() && audio_idx.is_none() {
        tracing::warn!("no video/audio track in input");
        return Err(Error::NoTrack);
    }

    // demuxer から時系列順にサンプルを取得し、トラック別に蓄積
    let mut video_samples: Vec<RawSample> = Vec::new();
    let mut audio_samples: Vec<RawSample> = Vec::new();
    loop {
        match demuxer.next_sample() {
            Ok(Some(sample)) => {
                let end_u64 = sample
                    .data_offset
                    .checked_add(sample.data_size as u64)
                    .ok_or_else(|| Error::Message("サンプルデータ範囲がオーバーフローしました".into()))?;
                let start = usize::try_from(sample.data_offset)
                    .map_err(|_| Error::Message("サンプルデータ開始位置が usize を超えました".into()))?;
                let end = usize::try_from(end_u64)
                    .map_err(|_| Error::Message("サンプルデータ終了位置が usize を超えました".into()))?;
                if end > input_data.len() {
                    // 破損入力の兆候として warn ログを出力する
                    tracing::warn!("sample data out of range: {start}..{end}, file size {}", input_data.len());
                    return Err(Error::InvalidSampleRange {
                        start,
                        end,
                        file_size: input_data.len(),
                    });
                }
                let data = input_data[start..end].to_vec();
                let raw = RawSample {
                    data,
                    timestamp: sample.timestamp,
                    duration: sample.duration,
                    keyframe: sample.keyframe,
                    composition_time_offset: sample.composition_time_offset,
                    sample_entry: sample.sample_entry.cloned(),
                };
                if sample.track.kind == TrackKind::Video {
                    video_samples.push(raw);
                } else if sample.track.kind == TrackKind::Audio {
                    audio_samples.push(raw);
                }
            }
            Ok(None) => break,
            Err(e) => {
                tracing::error!(error = %e, "demux error");
                return Err(e.into());
            }
        }
    }

    let video_timescale = video_idx.map(|i| tracks[i].timescale);
    let audio_timescale = audio_idx.map(|i| tracks[i].timescale);

    // 入力サンプルから実ビットレートを計算する (元の画質/音質を維持するため)
    let video_input_bitrate_kbps = if let Some(ts) = video_timescale {
        average_bitrate_kbps(&video_samples, ts.get())
    } else {
        None
    };
    let audio_input_bitrate_kbps = if let Some(ts) = audio_timescale {
        average_bitrate_kbps(&audio_samples, ts.get())
    } else {
        None
    };
    let recipe = recipe.resolve_input_bitrates(video_input_bitrate_kbps, audio_input_bitrate_kbps);

    // 進捗計算の基準として映像フレーム数 + 音声フレーム数 (概算) をセットする
    // 音声なしの場合は video_samples.len() だけ、映像なしの場合は audio_samples.len() で代替
    progress.set_total(video_samples.len().max(audio_samples.len()) as u64);

    // 映像と音声をエンコード
    // 音声の timescale は出力サンプルレートを使う (timestamp がサンプル単位のため) なので
    // ここでは入力の audio_timescale を渡すが encode_audio 側では使わない
    let encode_output = encode_tracks(
        &video_samples,
        &audio_samples,
        video_timescale,
        audio_timescale,
        recipe,
        progress,
    );
    // 成功したトラックだけを取り出す。所有権を move する。
    let video_for_mux = match encode_output.video {
        Ok(Some(v)) => Some(v),
        _ => None,
    };
    let audio_for_mux = match encode_output.audio {
        Ok(Some(a)) => Some(a),
        _ => None,
    };
    // 両方とも出力不能なら早期 return (空入力または両方失敗)
    if video_for_mux.is_none() && audio_for_mux.is_none() {
        return Ok(());
    }
    // 上書き不可設定かつ出力ファイルが既存の場合は Err を返す
    if !overwrite && output.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("output file already exists: {}", output.display()),
        )));
    }
    if let Err(e) = write_mp4(output, video_for_mux, audio_for_mux, video_timescale) {
        tracing::error!(error = %e, "mux error");
        return Err(e);
    }
    tracing::info!("encode finished");
    Ok(())
}

#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
type TrackEncodeResult<T> = std::result::Result<Option<T>, Error>;

#[cfg(target_os = "macos")]
struct EncodeTracksOutput {
    video: TrackEncodeResult<VideoOutput>,
    audio: TrackEncodeResult<AudioOutput>,
}

#[cfg(target_os = "macos")]
fn encode_tracks(
    video_samples: &[RawSample],
    audio_samples: &[RawSample],
    video_timescale: Option<NonZeroU32>,
    audio_timescale: Option<NonZeroU32>,
    recipe: EncodeRecipe,
    progress: &JobProgress,
) -> EncodeTracksOutput {
    let video = if !video_samples.is_empty() {
        match encode_video(
            video_samples,
            video_timescale,
            recipe.video,
            recipe.video_bitrate_kbps,
            progress,
        ) {
            Ok(output) => Ok(Some(output)),
            Err(e) => {
                tracing::warn!("video encode failed: {e}");
                Err(Error::TrackEncode {
                    track: "video",
                    message: e.to_string(),
                })
            }
        }
    } else {
        Ok(None)
    };

    let audio = if !audio_samples.is_empty() {
        match encode_audio(
            audio_samples,
            audio_timescale,
            recipe.audio,
            recipe.audio_bitrate_kbps,
            progress,
        ) {
            Ok(output) => Ok(Some(output)),
            Err(e) => {
                tracing::warn!("audio encode failed: {e}");
                Err(Error::TrackEncode {
                    track: "audio",
                    message: e.to_string(),
                })
            }
        }
    } else {
        Ok(None)
    };

    EncodeTracksOutput { video, audio }
}

#[cfg(not(target_os = "macos"))]
struct EncodeTracksOutput {
    video: Result<Option<()>, ()>,
    audio: Result<Option<()>, ()>,
}

#[cfg(not(target_os = "macos"))]
fn encode_tracks(
    _video_samples: &[RawSample],
    _audio_samples: &[RawSample],
    _video_timescale: Option<NonZeroU32>,
    _audio_timescale: Option<NonZeroU32>,
    _recipe: EncodeRecipe,
    _progress: &JobProgress,
) -> EncodeTracksOutput {
    EncodeTracksOutput {
        video: Ok(None),
        audio: Ok(None),
    }
}

// 以降の映像/音声/mux 関連関数は各サブモジュールに分割済み
