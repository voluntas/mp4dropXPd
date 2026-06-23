use std::num::NonZeroU32;

use shiguredo_mp4::boxes::SampleEntry;

#[cfg(target_os = "macos")]
use shiguredo_video_toolbox::{
    CodecConfig as VtCodecConfig, DecodedFrame as VtDecodedFrame, Decoder as VtDecoder,
    DecoderCodec as VtDecoderCodec, Encoder as VtEncoder, FrameData as VtFrameData,
    H264EntropyMode, H264Profile, HevcEncoderConfig, HevcProfile, PixelFormat as VtPixelFormat,
};

use crate::codec::VideoCodec;
use crate::encode::JobProgress;
use crate::encode::sample_entry::{
    build_av01_sample_entry, build_avc1_sample_entry, build_hev1_sample_entry,
};
use crate::error::{Error, Result};

use super::{
    EncodedVideoSample, RawSample, VideoOutput, DEFAULT_VIDEO_TIMESCALE,
    copy_stride, first_sample_entry, resolution_of, average_duration,
    extract_h265_params,
};
pub(crate) fn make_vt_decoder(entry: &SampleEntry) -> Result<VtDecoder> {
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
                // `?` で Err を呼び出し元へ即時伝播する。
                // 破損 dav1d デコーダで後続サンプルを処理し続けるリスクを排除するため。
                while let Some(frame) = decoder
                    .next_frame()
                    .map_err(|e| Error::Message(format!("dav1d next_frame error: {e}")))?
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
            // 全サンプル処理後、dav1d 内部バッファに残った遅延フレームを
            // `next_frame()` を `Ok(None)` まで反復呼び出ししてドレインする。
            // `Decoder::flush()` は内部状態リセット + バッファ破棄 API のため使用不可。
            // `Decoder::finish()` は no-op。
            // AudioToolbox デコードパターン (transcode.rs:716-724) を踏襲。
            let mut drained = 0usize;
            while let Some(frame) = decoder
                .next_frame()
                .map_err(|e| Error::Message(format!("dav1d drain error: {e}")))?
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
                drained += 1;
            }
            tracing::debug!("dav1d drain complete: {} delayed frames", drained);
            Ok(())
        }
        _ => Err(Error::Message("未対応の映像コーデックです".into())),
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn encode_video(
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
pub(crate) fn encode_video_h264(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
    progress: &JobProgress,
) -> Result<VideoOutput> {
    let first_entry = first_sample_entry(samples)?;
    let (width, height) = resolution_of(first_entry)?;
    let codec = VtCodecConfig::H264(shiguredo_video_toolbox::H264EncoderConfig {
        profile: H264Profile::Main,
        entropy_mode: H264EntropyMode::Cabac,
    });
    let encoded_frames = encode_video_vt(samples, width, height, timescale, bitrate_kbps, codec)?;
    progress.add_processed(encoded_frames.len() as u64);

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
    Ok(VideoOutput { samples: encoded, sample_entry })
}

#[cfg(target_os = "macos")]
pub(crate) fn encode_video_h265(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
    progress: &JobProgress,
) -> Result<VideoOutput> {
    let first_entry = first_sample_entry(samples)?;
    let (width, height) = resolution_of(first_entry)?;
    let codec = VtCodecConfig::Hevc(HevcEncoderConfig {
        profile: HevcProfile::Main,
        allow_open_gop: true,
    });
    let encoded_frames = encode_video_vt(samples, width, height, timescale, bitrate_kbps, codec)?;
    progress.add_processed(encoded_frames.len() as u64);

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
    Ok(VideoOutput { samples: encoded, sample_entry })
}

/// H.264 / H.265 の VideoToolbox エンコード共通処理
#[cfg(target_os = "macos")]
fn encode_video_vt(
    samples: &[RawSample],
    width: u16,
    height: u16,
    timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
    codec: VtCodecConfig,
) -> Result<Vec<shiguredo_video_toolbox::EncodedFrame>> {
    let ts = timescale.unwrap_or(DEFAULT_VIDEO_TIMESCALE);
    let avg_duration = average_duration(samples) as u32;
    let fps_denominator = if avg_duration == 0 { 1 } else { avg_duration };

    let mut encoder = VtEncoder::new(shiguredo_video_toolbox::EncoderConfig {
        width: width as u32,
        height: height as u32,
        codec,
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
        Ok(())
    })?;

    encoder
        .finish()
        .map_err(|e| Error::Message(format!("video encode finish error: {e}")))?;

    drain_vt_encoder(&mut encoder, samples.len())
}

#[cfg(target_os = "macos")]
/// `drain_vt_encoder` が `Ok(None)` を連続して受け取ったまま
/// 待機してよい最大の待ち時間
const DRAIN_VT_ENCODER_STALL_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(30);

#[cfg(target_os = "macos")]
pub(crate) fn drain_vt_encoder(
    encoder: &mut VtEncoder,
    expected: usize,
) -> Result<Vec<shiguredo_video_toolbox::EncodedFrame>> {
    let mut encoded_frames: Vec<shiguredo_video_toolbox::EncodedFrame> = Vec::new();
    let mut last_progress = std::time::Instant::now();
    while encoded_frames.len() < expected {
        match encoder.next_frame() {
            Ok(Some(frame)) => {
                // フレームが届いた → ストール判定をリセット
                last_progress = std::time::Instant::now();
                encoded_frames.push(frame);
            }
            Ok(None) => {
                // フレームがまだ届いていない。連続ストール判定。
                let elapsed = last_progress.elapsed();
                if elapsed >= DRAIN_VT_ENCODER_STALL_TIMEOUT {
                    let received = encoded_frames.len();
                    tracing::warn!(
                        elapsed = ?elapsed,
                        expected,
                        received,
                        "video encoder stalled"
                    );
                    return Err(Error::DrainTimeout {
                        elapsed: format!("{elapsed:?}"),
                        expected,
                        received,
                    });
                }
                // スリープで CPU を譲りつつポーリング間隔を確保
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
pub(crate) fn encode_video_av1(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    bitrate_kbps: u32,
    progress: &JobProgress,
) -> Result<VideoOutput> {
    let first_entry = first_sample_entry(samples)?;
    let (width, height) = resolution_of(first_entry)?;

    let ts = timescale.unwrap_or(DEFAULT_VIDEO_TIMESCALE);
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
    // 進捗はエンコード実完了ベースでカウントする
    progress.add_processed(encoded.len() as u64);

    // AV1 の SampleEntry は extra_data (OBU sequence header) を使う
    let sample_entry = build_av01_sample_entry(&extra_data, width, height);

    Ok(VideoOutput {
        samples: encoded,
        sample_entry,
    })
}

#[cfg(target_os = "macos")]
pub(crate) fn build_video_samples(
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
