use std::num::NonZeroU32;

use shiguredo_mp4::boxes::SampleEntry;

use crate::error::{Error, Result};

use super::RawSample;

/// デフォルトの映像タイムスケール (30 fps)
pub(crate) const DEFAULT_VIDEO_TIMESCALE: NonZeroU32 = NonZeroU32::new(30).expect("30 != 0");

pub(crate) fn first_sample_entry(samples: &[RawSample]) -> Result<&SampleEntry> {
    samples
        .iter()
        .find_map(|s| s.sample_entry.as_ref())
        .ok_or_else(|| Error::Message("トラックに SampleEntry がありません".into()))
}

#[cfg(target_os = "macos")]
pub(crate) fn resolution_of(entry: &SampleEntry) -> Result<(u16, u16)> {
    entry
        .video_resolution()
        .ok_or_else(|| Error::Message("映像の解像度を取得できません".into()))
}

#[cfg(target_os = "macos")]
pub(crate) fn audio_params_of(entry: &SampleEntry) -> Result<(u32, u8)> {
    entry
        .audio_sample_rate()
        .zip(entry.audio_channel_count())
        .map(|(r, c)| (r as u32, c))
        .ok_or_else(|| Error::Message("音声のサンプルレート/チャンネル数を取得できません".into()))
}

#[cfg(target_os = "macos")]
pub(crate) fn extract_h265_params(entry: &SampleEntry) -> Result<(&[u8], &[u8], &[u8])> {
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
pub(crate) fn copy_stride(src: &[u8], stride: usize, width: usize, height: usize) -> Vec<u8> {
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

pub(crate) fn average_duration(samples: &[RawSample]) -> u64 {
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
pub(crate) fn average_bitrate_kbps(samples: &[RawSample], timescale: u32) -> Option<u32> {
    if samples.is_empty() || timescale == 0 {
        return None;
    }
    let total_bytes: u64 = samples.iter().map(|s| s.data.len() as u64).sum();
    let total_duration_units: u64 = samples.iter().map(|s| s.duration as u64).sum();
    if total_duration_units == 0 {
        return None;
    }
        // ビットレート (bps) = 合計バイト数 × 8 × タイムスケール / 合計期間単位
    let bps = total_bytes.checked_mul(8)?.checked_mul(timescale as u64)? / total_duration_units;
    Some((bps / 1000) as u32)
}
