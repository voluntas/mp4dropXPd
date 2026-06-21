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
use std::io::{Seek, SeekFrom, Write};
use std::num::NonZeroU32;
use std::path::Path;

use shiguredo_mp4::TrackKind;
use shiguredo_mp4::boxes::SampleEntry;
use shiguredo_mp4::demux::{Input, Mp4FileDemuxer, TrackInfo};
use shiguredo_mp4::mux::{Mp4FileMuxer, Sample as MuxSample};

#[cfg(target_os = "macos")]
use shiguredo_audio_toolbox::{
    Decoder as AtDecoder, DecoderCodec as AtDecoderCodec, Encoder as AtEncoder,
    EncoderCodec as AtEncoderCodec,
};
#[cfg(target_os = "macos")]
use shiguredo_video_toolbox::{
    CodecConfig as VtCodecConfig, DecodedFrame as VtDecodedFrame, Decoder as VtDecoder,
    DecoderCodec as VtDecoderCodec, Encoder as VtEncoder, FrameData as VtFrameData,
    H264EntropyMode, H264Profile, HevcEncoderConfig, HevcProfile, PixelFormat as VtPixelFormat,
};

use crate::codec::{AudioCodec, EncodeRecipe, VideoCodec};
use crate::encode::JobProgress;
use crate::encode::sample_entry::{
    build_av01_sample_entry, build_avc1_sample_entry, build_hev1_sample_entry,
    build_mp4a_sample_entry, build_opus_sample_entry,
};
use crate::error::{Error, Result};

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
    progress: &JobProgress,
) -> Result<()> {
    let input_data = std::fs::read(input)?;

    let mut demuxer = Mp4FileDemuxer::new();
    demuxer.handle_input(Input {
        position: 0,
        data: &input_data,
    });
    let tracks: Vec<TrackInfo> = demuxer.tracks()?.to_vec();

    let video_idx = tracks.iter().position(|t| t.kind == TrackKind::Video);
    let audio_idx = tracks.iter().position(|t| t.kind == TrackKind::Audio);

    // demuxer から時系列順にサンプルを取得し、トラック別に蓄積
    let mut video_samples: Vec<RawSample> = Vec::new();
    let mut audio_samples: Vec<RawSample> = Vec::new();
    loop {
        match demuxer.next_sample() {
            Ok(Some(sample)) => {
                let data = input_data
                    [sample.data_offset as usize..sample.data_offset as usize + sample.data_size]
                    .to_vec();
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
            Err(e) => return Err(Error::Message(format!("demux error: {e}"))),
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
    let recipe = recipe.with_input_bitrates(video_input_bitrate_kbps, audio_input_bitrate_kbps);

    // 進捗計算の基準として映像フレーム数をセットする
    // 音声は映像に比べて処理時間が短いため、映像フレームを基準とする
    progress.set_total(video_samples.len() as u64);

    // 映像と音声をエンコード
    // 音声の timescale は出力サンプルレートを使う (timestamp がサンプル単位のため) なので
    // ここでは入力の audio_timescale を渡すが encode_audio 側では使わない
    let (video_output, audio_output) = encode_tracks(
        &video_samples,
        &audio_samples,
        video_timescale,
        audio_timescale,
        recipe,
        progress,
    )?;

    // mux して出力ファイルに書き込み
    // 音声の timescale は AudioOutput.sample_rate から取り出すので渡さない
    write_mp4(output, video_output, audio_output, video_timescale)
}

#[cfg(target_os = "macos")]
fn encode_tracks(
    video_samples: &[RawSample],
    audio_samples: &[RawSample],
    video_timescale: Option<NonZeroU32>,
    audio_timescale: Option<NonZeroU32>,
    recipe: EncodeRecipe,
    progress: &JobProgress,
) -> Result<(Option<VideoOutput>, Option<AudioOutput>)> {
    let video_output = if !video_samples.is_empty() {
        Some(encode_video(
            video_samples,
            video_timescale,
            recipe.video,
            recipe.video_bitrate_kbps,
            progress,
        )?)
    } else {
        None
    };

    let audio_output = if !audio_samples.is_empty() {
        Some(encode_audio(
            audio_samples,
            audio_timescale,
            recipe.audio,
            recipe.audio_bitrate_kbps,
        )?)
    } else {
        None
    };

    Ok((video_output, audio_output))
}

#[cfg(not(target_os = "macos"))]
fn encode_tracks(
    _video_samples: &[RawSample],
    _audio_samples: &[RawSample],
    _video_timescale: Option<NonZeroU32>,
    _audio_timescale: Option<NonZeroU32>,
    _recipe: EncodeRecipe,
    _progress: &JobProgress,
) -> Result<(Option<VideoOutput>, Option<AudioOutput>)> {
    Err(Error::Message(
        "mp4dropXPd は macOS 以外ではエンコードできません".into(),
    ))
}

// ===== 映像エンコード =====

/// 入力 SampleEntry から VideoToolbox デコーダを生成する (H.264/H.265)
#[cfg(target_os = "macos")]
fn make_vt_decoder(entry: &SampleEntry) -> Result<VtDecoder> {
    match entry {
        SampleEntry::Avc1(avc1) => {
            let sps = avc1
                .avcc_box
                .sps_list
                .first()
                .ok_or_else(|| Error::Message("avcC に SPS がありません".into()))?;
            let pps = avc1
                .avcc_box
                .pps_list
                .first()
                .ok_or_else(|| Error::Message("avcC に PPS がありません".into()))?;
            VtDecoder::new(shiguredo_video_toolbox::DecoderConfig {
                codec: VtDecoderCodec::H264 {
                    sps,
                    pps,
                    nalu_len_bytes: 4,
                },
                pixel_format: VtPixelFormat::I420,
            })
            .map_err(Into::into)
        }
        SampleEntry::Hev1(_) | SampleEntry::Hvc1(_) => {
            let (vps, sps, pps) = extract_h265_params(entry)?;
            VtDecoder::new(shiguredo_video_toolbox::DecoderConfig {
                codec: VtDecoderCodec::Hevc {
                    vps,
                    sps,
                    pps,
                    nalu_len_bytes: 4,
                },
                pixel_format: VtPixelFormat::I420,
            })
            .map_err(Into::into)
        }
        _ => Err(Error::Message(
            "VideoToolbox でデコードできない映像コーデックです".into(),
        )),
    }
}

/// 入力 SampleEntry に応じて dav1d または VideoToolbox デコーダを選択し、
/// すべてのサンプルを I420 フレームとして順にエンコードに渡すコールバックを呼ぶ
#[cfg(target_os = "macos")]
fn decode_video_frames(
    samples: &[RawSample],
    width: u16,
    height: u16,
    mut on_frame: impl FnMut(&[u8], &[u8], &[u8]) -> Result<()>,
) -> Result<()> {
    let first_entry = first_sample_entry(samples)?;
    let w = width as usize;
    let h = height as usize;

    match first_entry {
        SampleEntry::Avc1(_) | SampleEntry::Hev1(_) | SampleEntry::Hvc1(_) => {
            let mut decoder = make_vt_decoder(first_entry)?;
            for raw in samples {
                let decoded = decoder
                    .decode(&raw.data)
                    .map_err(|e| Error::Message(format!("video decode error: {e}")))?;
                let Some(frame) = decoded else {
                    continue;
                };
                let i420 = match frame {
                    VtDecodedFrame::I420(ref f) => f,
                    VtDecodedFrame::Nv12(_) => {
                        return Err(Error::Message(
                            "NV12 デコード出力には対応していません (I420 を期待)".into(),
                        ));
                    }
                };
                let y = copy_stride(i420.y_plane(), i420.y_stride(), w, h);
                let u = copy_stride(i420.u_plane(), i420.u_stride(), w / 2, h / 2);
                let v = copy_stride(i420.v_plane(), i420.v_stride(), w / 2, h / 2);
                on_frame(&y, &u, &v)?;
            }
            Ok(())
        }
        SampleEntry::Av01(_) => {
            let mut decoder =
                shiguredo_dav1d::Decoder::new(shiguredo_dav1d::DecoderConfig::default())
                    .map_err(|e| Error::Message(format!("dav1d init error: {e}")))?;
            for raw in samples {
                decoder
                    .decode(&raw.data)
                    .map_err(|e| Error::Message(format!("dav1d decode error: {e}")))?;
                while let Ok(Some(frame)) = decoder
                    .next_frame()
                    .map_err(|e| Error::Message(format!("dav1d next_frame error: {e}")))
                {
                    if frame.bit_depth() != 8 {
                        return Err(Error::Message(
                            "AV1 デコード結果が 8-bit ではありません (現状は 8-bit I420 のみ対応)"
                                .into(),
                        ));
                    }
                    let y = copy_stride(frame.y_plane(), frame.y_stride(), w, h);
                    let u = copy_stride(frame.u_plane(), frame.u_stride(), w / 2, h / 2);
                    let v = copy_stride(frame.v_plane(), frame.v_stride(), w / 2, h / 2);
                    on_frame(&y, &u, &v)?;
                }
            }
            Ok(())
        }
        _ => Err(Error::Message("未対応の映像コーデックです".into())),
    }
}

#[cfg(target_os = "macos")]
fn encode_video(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    video_codec: VideoCodec,
    bitrate_kbps: u32,
    progress: &JobProgress,
) -> Result<VideoOutput> {
    match video_codec {
        VideoCodec::H264 => encode_video_h264(samples, timescale, bitrate_kbps, progress),
        VideoCodec::H265 => encode_video_h265(samples, timescale, bitrate_kbps, progress),
        VideoCodec::Av1 => encode_video_av1(samples, timescale, bitrate_kbps, progress),
    }
}

#[cfg(target_os = "macos")]
fn encode_video_h264(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
    progress: &JobProgress,
) -> Result<VideoOutput> {
    let first_entry = first_sample_entry(samples)?;
    let (width, height) = resolution_of(first_entry)?;

    let ts = timescale.unwrap_or_else(|| NonZeroU32::new(30).expect("30 != 0"));
    let avg_duration = average_duration(samples) as u32;
    let fps_denominator = if avg_duration == 0 { 1 } else { avg_duration };

    let mut encoder = VtEncoder::new(shiguredo_video_toolbox::EncoderConfig {
        width: width as u32,
        height: height as u32,
        codec: VtCodecConfig::H264(shiguredo_video_toolbox::H264EncoderConfig {
            profile: H264Profile::Main,
            entropy_mode: H264EntropyMode::Cabac,
        }),
        pixel_format: VtPixelFormat::I420,
        average_bitrate: Some(bitrate_kbps as u64 * 1000),
        fps_numerator: ts.get(),
        fps_denominator,
        prioritize_encoding_speed_over_quality: false,
        real_time: false,
        maximize_power_efficiency: false,
        allow_frame_reordering: false,
        allow_temporal_compression: true,
        max_key_frame_interval: None,
        max_key_frame_interval_duration: None,
        // フレーム遅延を 1 に制限して、エンコード完了後にフレームが届かない
        // (ドロップ/無限待ち) 状態を防ぐ
        max_frame_delay_count: NonZeroU32::new(1),
    })?;

    // デコード → エンコード
    let mut raw_iter = samples.iter();
    decode_video_frames(samples, width, height, |y, u, v| {
        let raw = raw_iter.next();
        encoder
            .encode(
                &VtFrameData::I420 { y, u, v },
                &shiguredo_video_toolbox::EncodeOptions {
                    force_key_frame: raw.is_some_and(|r| r.keyframe),
                },
            )
            .map_err(|e| Error::Message(format!("video encode error: {e}")))?;
        progress.add_processed(1);
        Ok(())
    })?;

    encoder
        .finish()
        .map_err(|e| Error::Message(format!("video encode finish error: {e}")))?;

    // finish 後に全フレームを取り出す
    let encoded_frames = drain_vt_encoder(&mut encoder, samples.len())?;

    // 最初のキーフレームから SPS/PPS を取得
    let (enc_sps, enc_pps) = encoded_frames
        .iter()
        .find(|f| f.keyframe)
        .and_then(|f| {
            let sps = f.sps_list.first()?;
            let pps = f.pps_list.first()?;
            Some((sps.clone(), pps.clone()))
        })
        .ok_or_else(|| {
            Error::Message("H.264 エンコード結果から SPS/PPS を取得できません".into())
        })?;

    let encoded = build_video_samples(&encoded_frames, samples);
    let sample_entry = build_avc1_sample_entry(&enc_sps, &enc_pps, width, height);

    Ok(VideoOutput {
        samples: encoded,
        sample_entry,
    })
}

#[cfg(target_os = "macos")]
fn encode_video_h265(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
    progress: &JobProgress,
) -> Result<VideoOutput> {
    let first_entry = first_sample_entry(samples)?;
    let (width, height) = resolution_of(first_entry)?;

    let ts = timescale.unwrap_or_else(|| NonZeroU32::new(30).expect("30 != 0"));
    let avg_duration = average_duration(samples) as u32;
    let fps_denominator = if avg_duration == 0 { 1 } else { avg_duration };

    let mut encoder = VtEncoder::new(shiguredo_video_toolbox::EncoderConfig {
        width: width as u32,
        height: height as u32,
        codec: VtCodecConfig::Hevc(HevcEncoderConfig {
            profile: HevcProfile::Main,
            allow_open_gop: true,
        }),
        pixel_format: VtPixelFormat::I420,
        average_bitrate: Some(bitrate_kbps as u64 * 1000),
        fps_numerator: ts.get(),
        fps_denominator,
        prioritize_encoding_speed_over_quality: false,
        real_time: false,
        maximize_power_efficiency: false,
        allow_frame_reordering: false,
        allow_temporal_compression: true,
        max_key_frame_interval: None,
        max_key_frame_interval_duration: None,
        // フレーム遅延を 1 に制限して、エンコード完了後にフレームが届かない
        // (ドロップ/無限待ち) 状態を防ぐ
        max_frame_delay_count: NonZeroU32::new(1),
    })?;

    // デコード → エンコード
    let mut raw_iter = samples.iter();
    decode_video_frames(samples, width, height, |y, u, v| {
        let raw = raw_iter.next();
        encoder
            .encode(
                &VtFrameData::I420 { y, u, v },
                &shiguredo_video_toolbox::EncodeOptions {
                    force_key_frame: raw.is_some_and(|r| r.keyframe),
                },
            )
            .map_err(|e| Error::Message(format!("video encode error: {e}")))?;
        progress.add_processed(1);
        Ok(())
    })?;

    encoder
        .finish()
        .map_err(|e| Error::Message(format!("video encode finish error: {e}")))?;

    let encoded_frames = drain_vt_encoder(&mut encoder, samples.len())?;

    // H.265 の場合、EncodedFrame から VPS/SPS/PPS を取得
    let (enc_vps, enc_sps, enc_pps) = encoded_frames
        .iter()
        .find(|f| f.keyframe)
        .and_then(|f| {
            let vps = f.vps_list.first()?;
            let sps = f.sps_list.first()?;
            let pps = f.pps_list.first()?;
            Some((vps.clone(), sps.clone(), pps.clone()))
        })
        .ok_or_else(|| {
            Error::Message("H.265 エンコード結果から VPS/SPS/PPS を取得できません".into())
        })?;

    let encoded = build_video_samples(&encoded_frames, samples);
    let sample_entry = build_hev1_sample_entry(&enc_vps, &enc_sps, &enc_pps, width, height);

    Ok(VideoOutput {
        samples: encoded,
        sample_entry,
    })
}

#[cfg(target_os = "macos")]
fn drain_vt_encoder(
    encoder: &mut VtEncoder,
    expected: usize,
) -> Result<Vec<shiguredo_video_toolbox::EncodedFrame>> {
    // VideoToolbox のエンコードは非同期で、finish() 後にフレームが順次届く。
    // 以前は empty_retries > 1000 (合計 100ms) で打ち切っていたが、
    // エンコーダ内部の遅延が大きいとフレームを欠落させる原因になるため、
    // 期待フレーム数に達するまで無制限に待つ。
    let mut encoded_frames: Vec<shiguredo_video_toolbox::EncodedFrame> = Vec::new();
    while encoded_frames.len() < expected {
        match encoder.next_frame() {
            Ok(Some(frame)) => {
                encoded_frames.push(frame);
            }
            Ok(None) => {
                // フレームがまだ届いていないだけなので短いスリープで待つ
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
            Err(e) => {
                return Err(Error::Message(format!("video next_frame error: {e}")));
            }
        }
    }
    Ok(encoded_frames)
}

#[cfg(target_os = "macos")]
fn encode_video_av1(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
    progress: &JobProgress,
) -> Result<VideoOutput> {
    let first_entry = first_sample_entry(samples)?;
    let (width, height) = resolution_of(first_entry)?;

    let ts = timescale.unwrap_or_else(|| NonZeroU32::new(30).expect("30 != 0"));
    let avg_duration = average_duration(samples) as u32;
    let fps_denominator = if avg_duration == 0 { 1 } else { avg_duration };

    // SVT-AV1 エンコーダ
    let mut encoder = shiguredo_svt_av1::Encoder::new({
        let mut cfg = shiguredo_svt_av1::EncoderConfig::new(
            width as usize,
            height as usize,
            shiguredo_svt_av1::ColorFormat::I420,
        );
        cfg.fps_numerator = ts.get() as usize;
        cfg.fps_denominator = fps_denominator as usize;
        cfg.target_bit_rate = (bitrate_kbps as usize) * 1000;
        cfg.rate_control_mode = shiguredo_svt_av1::RcMode::Vbr;
        cfg.enc_mode = 8;
        cfg
    })
    .map_err(|e| Error::Message(format!("svt_av1 init error: {e}")))?;

    let extra_data = encoder.extra_data().to_vec();

    let mut encoded: Vec<EncodedVideoSample> = Vec::new();
    let mut raw_iter = samples.iter();

    // デコード → エンコード
    // SVT-AV1 の PTS は入力順に 0,1,2,... と振られる
    decode_video_frames(samples, width, height, |y, u, v| {
        let raw = raw_iter.next();
        encoder
            .encode(
                &shiguredo_svt_av1::FrameData::I420 { y, u, v },
                &shiguredo_svt_av1::EncodeOptions {
                    force_keyframe: raw.is_some_and(|r| r.keyframe),
                },
            )
            .map_err(|e| Error::Message(format!("svt_av1 encode error: {e}")))?;
        progress.add_processed(1);

        // エンコード結果を取り出す
        while let Some(enc_frame) = encoder.next_frame() {
            let idx = enc_frame.pts() as usize;
            let (timestamp, duration, cto) = samples
                .get(idx)
                .map(|s| (s.timestamp, s.duration, s.composition_time_offset))
                .unwrap_or((0, 1, None));
            encoded.push(EncodedVideoSample {
                data: enc_frame.data().to_vec(),
                keyframe: enc_frame.is_keyframe(),
                timestamp,
                duration,
                composition_time_offset: cto,
            });
        }
        Ok(())
    })?;

    // エンコーダの残りをフラッシュ
    encoder
        .finish()
        .map_err(|e| Error::Message(format!("svt_av1 finish error: {e}")))?;
    while let Some(enc_frame) = encoder.next_frame() {
        let idx = enc_frame.pts() as usize;
        let (timestamp, duration, cto) = samples
            .get(idx)
            .map(|s| (s.timestamp, s.duration, s.composition_time_offset))
            .unwrap_or((0, 1, None));
        encoded.push(EncodedVideoSample {
            data: enc_frame.data().to_vec(),
            keyframe: enc_frame.is_keyframe(),
            timestamp,
            duration,
            composition_time_offset: cto,
        });
    }

    // PTS 順にソートする (SVT-AV1 はフレーム並び替えを行う場合がある)
    encoded.sort_by_key(|s| s.timestamp);

    // AV1 の SampleEntry は extra_data (OBU sequence header) を使う
    let sample_entry = build_av01_sample_entry(&extra_data, width, height);

    Ok(VideoOutput {
        samples: encoded,
        sample_entry,
    })
}

#[cfg(target_os = "macos")]
fn build_video_samples(
    encoded_frames: &[shiguredo_video_toolbox::EncodedFrame],
    raw_samples: &[RawSample],
) -> Vec<EncodedVideoSample> {
    // allow_frame_reordering: false なので入力順と出力順は一致する
    encoded_frames
        .iter()
        .enumerate()
        .map(|(i, frame)| {
            let raw = &raw_samples[i];
            EncodedVideoSample {
                data: frame.data.clone(),
                keyframe: frame.keyframe,
                timestamp: raw.timestamp,
                duration: raw.duration,
                composition_time_offset: raw.composition_time_offset,
            }
        })
        .collect()
}

// ===== 音声エンコード =====

#[cfg(target_os = "macos")]
fn encode_audio(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    audio_codec: AudioCodec,
    bitrate_kbps: u32,
) -> Result<AudioOutput> {
    match audio_codec {
        AudioCodec::Aac => encode_audio_aac(samples, timescale, bitrate_kbps),
        AudioCodec::Opus => encode_audio_opus(samples, timescale, bitrate_kbps),
    }
}

/// 入力 SampleEntry から音声デコーダを選択して生成する
#[cfg(target_os = "macos")]
fn make_audio_decoder(entry: &SampleEntry) -> Result<AtDecoder> {
    let (sample_rate, channels) = audio_params_of(entry)?;
    let codec = match entry {
        SampleEntry::Mp4a(_) => AtDecoderCodec::AacLc,
        SampleEntry::Opus(_) => AtDecoderCodec::Opus,
        _ => return Err(Error::Message("未対応の音声コーデックです".into())),
    };
    AtDecoder::new(shiguredo_audio_toolbox::DecoderConfig {
        codec,
        input_sample_rate: sample_rate,
        input_channels: channels,
    })
    .map_err(|e| Error::Message(format!("audio decoder init error: {e}")))
}

/// すべての入力サンプルを PCM にデコードして返す
///
/// `shiguredo_audio_toolbox::Decoder` は出力を **ステレオ (2ch) 固定** で返すため、
/// 戻り値のチャンネル数は入力チャンネル数に関わらず常に 2 になる。
/// エンコーダと SampleEntry の構築でもこの 2ch を使う必要がある。
#[cfg(target_os = "macos")]
fn decode_audio_to_pcm(samples: &[RawSample]) -> Result<(Vec<i16>, u32, u8)> {
    let first_entry = first_sample_entry(samples)?;
    let (input_sample_rate, _input_channels) = audio_params_of(first_entry)?;
    let mut decoder = make_audio_decoder(first_entry)?;
    let mut pcm: Vec<i16> = Vec::new();
    for raw in samples {
        decoder
            .decode(&raw.data)
            .map_err(|e| Error::Message(format!("audio decode error: {e}")))?;
        while let Some(frame) = decoder
            .next_frame()
            .map_err(|e| Error::Message(format!("audio decode next_frame error: {e}")))?
        {
            pcm.extend_from_slice(&frame);
        }
    }
    decoder
        .finish()
        .map_err(|e| Error::Message(format!("audio decode finish error: {e}")))?;
    while let Some(frame) = decoder
        .next_frame()
        .map_err(|e| Error::Message(format!("audio decode next_frame (after finish) error: {e}")))?
    {
        pcm.extend_from_slice(&frame);
    }
    // デコーダ出力はステレオ (2ch) 固定
    Ok((pcm, input_sample_rate, 2))
}

#[cfg(target_os = "macos")]
fn encode_audio_aac(
    samples: &[RawSample],
    _timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
) -> Result<AudioOutput> {
    let (pcm, sample_rate, channels) = decode_audio_to_pcm(samples)?;

    // エンコーダ (AAC)
    let mut encoder = AtEncoder::new(shiguredo_audio_toolbox::EncoderConfig {
        codec: AtEncoderCodec::AacLc,
        sample_rate,
        channels,
        bitrate: Some(bitrate_kbps * 1000),
        bitrate_control_mode: None,
        codec_quality: None,
        vbr_quality: None,
    })
    .map_err(|e| Error::Message(format!("audio encoder init error: {e}")))?;

    // PCM をエンコーダに送る
    encoder
        .encode(&pcm)
        .map_err(|e| Error::Message(format!("audio encode error: {e}")))?;
    encoder
        .finish()
        .map_err(|e| Error::Message(format!("audio encode finish error: {e}")))?;

    let mut encoded: Vec<EncodedAudioSample> = Vec::new();
    let mut cumulative_samples: u64 = 0;
    while let Some(frame) = encoder.next_frame() {
        let duration = frame.samples as u32;
        encoded.push(EncodedAudioSample {
            data: frame.data,
            timestamp: cumulative_samples,
            duration,
        });
        cumulative_samples += duration as u64;
    }

    let sample_entry = build_mp4a_sample_entry(sample_rate, channels);

    Ok(AudioOutput {
        samples: encoded,
        sample_entry,
        sample_rate,
    })
}

#[cfg(target_os = "macos")]
fn encode_audio_opus(
    samples: &[RawSample],
    _timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
) -> Result<AudioOutput> {
    let (pcm, sample_rate, channels) = decode_audio_to_pcm(samples)?;

    // Opus がサポートするサンプルレートは 8000/12000/16000/24000/48000
    // 入力がこれら以外 (44100 等) の場合は線形リサンプリングして 48000Hz に変換する
    let opus_sample_rate: u32 = if [8000, 12000, 16000, 24000, 48000].contains(&sample_rate) {
        sample_rate
    } else {
        48000
    };
    let pcm = if opus_sample_rate == sample_rate {
        pcm
    } else {
        resample_linear(&pcm, sample_rate, opus_sample_rate, channels)
    };

    // エンコーダ (Opus)
    let mut opus_config = shiguredo_opus::EncoderConfig::new(opus_sample_rate, channels);
    opus_config.bitrate = Some(bitrate_kbps * 1000);
    let mut encoder = shiguredo_opus::Encoder::new(opus_config)
        .map_err(|e| Error::Message(format!("opus encoder init error: {e}")))?;

    let pre_skip = encoder
        .get_lookahead()
        .map_err(|e| Error::Message(format!("opus get_lookahead error: {e}")))?;

    let frame_samples = encoder.frame_samples();
    let needed = frame_samples * channels as usize;
    let mut encoded: Vec<EncodedAudioSample> = Vec::new();
    let mut cumulative_samples: u64 = 0;

    // PCM を frame_samples 単位でエンコード
    let mut i = 0;
    while i + needed <= pcm.len() {
        let chunk = &pcm[i..i + needed];
        let enc_data = encoder
            .encode(chunk)
            .map_err(|e| Error::Message(format!("opus encode error: {e}")))?;
        let duration = frame_samples as u32;
        encoded.push(EncodedAudioSample {
            data: enc_data,
            timestamp: cumulative_samples,
            duration,
        });
        cumulative_samples += duration as u64;
        i += needed;
    }

    // 残りの PCM が frame_samples に満たない場合はパディングしてエンコード
    if i < pcm.len() {
        let mut padded = pcm[i..].to_vec();
        padded.resize(needed, 0);
        let enc_data = encoder
            .encode(&padded)
            .map_err(|e| Error::Message(format!("opus encode (padded) error: {e}")))?;
        let duration = frame_samples as u32;
        encoded.push(EncodedAudioSample {
            data: enc_data,
            timestamp: cumulative_samples,
            duration,
        });
    }

    let sample_entry = build_opus_sample_entry(opus_sample_rate, channels, pre_skip);

    Ok(AudioOutput {
        samples: encoded,
        sample_entry,
        sample_rate: opus_sample_rate,
    })
}

/// 線形リサンプリングでサンプルレートを変換する
///
/// 入力 PCM はインターリーブ形式 (channels チャンネル) の i16。
/// 線形補間なので音質は高くないが、Opus エンコーダに入れる前段としては十分実用になる。
#[cfg(target_os = "macos")]
fn resample_linear(pcm: &[i16], from_hz: u32, to_hz: u32, channels: u8) -> Vec<i16> {
    if from_hz == to_hz || pcm.is_empty() {
        return pcm.to_vec();
    }
    let channels = channels as usize;
    let input_frames = pcm.len() / channels;
    if input_frames == 0 {
        return Vec::new();
    }
    // 出力フレーム数 = ceil(input_frames * to_hz / from_hz)
    let output_frames = (input_frames as u64 * to_hz as u64).div_ceil(from_hz as u64);
    let ratio = from_hz as f64 / to_hz as f64;
    let mut out = Vec::with_capacity(output_frames as usize * channels);
    for i in 0..output_frames {
        let src_pos = i as f64 * ratio;
        let idx0 = src_pos.floor() as usize;
        let idx1 = (idx0 + 1).min(input_frames - 1);
        let frac = src_pos - idx0 as f64;
        for c in 0..channels {
            let s0 = pcm[idx0 * channels + c] as f64;
            let s1 = pcm[idx1 * channels + c] as f64;
            let v = s0 + (s1 - s0) * frac;
            out.push(v.round().clamp(-32768.0, 32767.0) as i16);
        }
    }
    out
}

// ===== ヘルパー =====

#[cfg(target_os = "macos")]
fn first_sample_entry(samples: &[RawSample]) -> Result<&SampleEntry> {
    samples
        .iter()
        .find_map(|s| s.sample_entry.as_ref())
        .ok_or_else(|| Error::Message("トラックに SampleEntry がありません".into()))
}

#[cfg(target_os = "macos")]
fn resolution_of(entry: &SampleEntry) -> Result<(u16, u16)> {
    entry
        .video_resolution()
        .ok_or_else(|| Error::Message("映像の解像度を取得できません".into()))
}

#[cfg(target_os = "macos")]
fn audio_params_of(entry: &SampleEntry) -> Result<(u32, u8)> {
    entry
        .audio_sample_rate()
        .zip(entry.audio_channel_count())
        .map(|(r, c)| (r as u32, c))
        .ok_or_else(|| Error::Message("音声のサンプルレート/チャンネル数を取得できません".into()))
}

#[cfg(target_os = "macos")]
fn extract_h265_params(entry: &SampleEntry) -> Result<(&[u8], &[u8], &[u8])> {
    let hvcc = match entry {
        SampleEntry::Hev1(b) => &b.hvcc_box,
        SampleEntry::Hvc1(b) => &b.hvcc_box,
        _ => return Err(Error::Message("映像トラックが H.265 ではありません".into())),
    };

    let vps = hvcc
        .nalu_arrays
        .iter()
        .find(|a| a.nal_unit_type.get() == 32)
        .and_then(|a| a.nalus.first())
        .ok_or_else(|| Error::Message("hvcC に VPS がありません".into()))?;
    let sps = hvcc
        .nalu_arrays
        .iter()
        .find(|a| a.nal_unit_type.get() == 33)
        .and_then(|a| a.nalus.first())
        .ok_or_else(|| Error::Message("hvcC に SPS がありません".into()))?;
    let pps = hvcc
        .nalu_arrays
        .iter()
        .find(|a| a.nal_unit_type.get() == 34)
        .and_then(|a| a.nalus.first())
        .ok_or_else(|| Error::Message("hvcC に PPS がありません".into()))?;

    Ok((vps, sps, pps))
}

#[cfg(target_os = "macos")]
fn copy_stride(src: &[u8], stride: usize, width: usize, height: usize) -> Vec<u8> {
    if stride == width || height == 0 {
        return src[..width * height].to_vec();
    }
    let mut dst = Vec::new();
    for row in 0..height {
        let start = row * stride;
        dst.extend_from_slice(&src[start..start + width]);
    }
    dst
}

fn average_duration(samples: &[RawSample]) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let total: u64 = samples.iter().map(|s| s.duration as u64).sum();
    total / samples.len() as u64
}

/// サンプルデータの総バイト数と総デコード時間から平均ビットレート (kbps) を計算する
///
/// 入力 MP4 にビットレート情報が無くても、サンプルサイズと時間から正確な実ビットレートを求められる。
/// `timescale` はトラックの timescale (Hz)。
fn average_bitrate_kbps(samples: &[RawSample], timescale: u32) -> Option<u32> {
    if samples.is_empty() || timescale == 0 {
        return None;
    }
    let total_bytes: u64 = samples.iter().map(|s| s.data.len() as u64).sum();
    let total_duration_units: u64 = samples.iter().map(|s| s.duration as u64).sum();
    if total_duration_units == 0 {
        return None;
    }
    // bps = total_bytes * 8 * timescale / total_duration_units
    let bps = total_bytes.checked_mul(8)?.checked_mul(timescale as u64)? / total_duration_units;
    Some((bps / 1000) as u32)
}

// ===== MP4 出力 =====

#[cfg(target_os = "macos")]
fn write_mp4(
    output: &Path,
    video: Option<VideoOutput>,
    audio: Option<AudioOutput>,
    video_timescale: Option<NonZeroU32>,
) -> Result<()> {
    let mut muxer = Mp4FileMuxer::new().map_err(|e| Error::Message(format!("muxer init: {e}")))?;
    let initial_bytes = muxer.initial_boxes_bytes().to_vec();

    let mut file = std::fs::File::create(output)?;
    file.write_all(&initial_bytes)?;

    // 音声の timescale は出力サンプルレートを使う (timestamp がサンプル単位のため)
    let audio_timescale = audio.as_ref().and_then(|a| NonZeroU32::new(a.sample_rate));

    // 時系列順にマージするためのイベントリスト
    let mut events: Vec<(f64, OutputKind)> = Vec::new();

    if let Some(v) = &video {
        let ts = video_timescale.expect("video timescale");
        for s in &v.samples {
            let secs = s.timestamp as f64 / ts.get() as f64;
            events.push((
                secs,
                OutputKind::Video(EncodedVideoSample {
                    data: s.data.clone(),
                    keyframe: s.keyframe,
                    timestamp: s.timestamp,
                    duration: s.duration,
                    composition_time_offset: s.composition_time_offset,
                }),
            ));
        }
    }
    if let Some(a) = &audio {
        let ts = audio_timescale.expect("audio timescale");
        for s in &a.samples {
            let secs = s.timestamp as f64 / ts.get() as f64;
            events.push((
                secs,
                OutputKind::Audio(EncodedAudioSample {
                    data: s.data.clone(),
                    timestamp: s.timestamp,
                    duration: s.duration,
                }),
            ));
        }
    }
    events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let video_entry = video.as_ref().map(|v| v.sample_entry.clone());
    let audio_entry = audio.as_ref().map(|a| a.sample_entry.clone());
    let video_ts = video_timescale;
    let audio_ts = audio_timescale;

    let mut video_entry_sent = false;
    let mut audio_entry_sent = false;
    let mut position = initial_bytes.len() as u64;

    for (_, kind) in events {
        match kind {
            OutputKind::Video(s) => {
                file.write_all(&s.data)?;
                let data_size = s.data.len();
                let ts = video_ts.expect("video timescale");
                let sample = MuxSample {
                    track_kind: TrackKind::Video,
                    sample_entry: if video_entry_sent {
                        None
                    } else {
                        video_entry_sent = true;
                        video_entry.clone()
                    },
                    keyframe: s.keyframe,
                    timescale: ts,
                    duration: s.duration,
                    composition_time_offset: s.composition_time_offset,
                    data_offset: position,
                    data_size,
                };
                muxer
                    .append_sample(&sample)
                    .map_err(|e| Error::Message(format!("mux append video: {e}")))?;
                position += data_size as u64;
            }
            OutputKind::Audio(s) => {
                file.write_all(&s.data)?;
                let data_size = s.data.len();
                let ts = audio_ts.expect("audio timescale");
                let sample = MuxSample {
                    track_kind: TrackKind::Audio,
                    sample_entry: if audio_entry_sent {
                        None
                    } else {
                        audio_entry_sent = true;
                        audio_entry.clone()
                    },
                    keyframe: false,
                    timescale: ts,
                    duration: s.duration,
                    composition_time_offset: None,
                    data_offset: position,
                    data_size,
                };
                muxer
                    .append_sample(&sample)
                    .map_err(|e| Error::Message(format!("mux append audio: {e}")))?;
                position += data_size as u64;
            }
        }
    }

    // finalize
    let finalized = muxer
        .finalize()
        .map_err(|e| Error::Message(format!("mux finalize: {e}")))?;
    for (offset, bytes) in finalized.offset_and_bytes_pairs() {
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(bytes)?;
    }
    file.flush()?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn write_mp4(
    _output: &Path,
    _video: Option<VideoOutput>,
    _audio: Option<AudioOutput>,
    _video_timescale: Option<NonZeroU32>,
) -> Result<()> {
    Err(Error::Message(
        "mp4dropXPd は macOS 以外ではエンコードできません".into(),
    ))
}
