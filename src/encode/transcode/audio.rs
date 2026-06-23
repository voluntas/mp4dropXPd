use std::num::NonZeroU32;

use shiguredo_mp4::boxes::SampleEntry;

#[cfg(target_os = "macos")]
use shiguredo_audio_toolbox::{
    Decoder as AtDecoder, DecoderCodec as AtDecoderCodec, Encoder as AtEncoder,
    EncoderCodec as AtEncoderCodec,
};

use crate::codec::AudioCodec;
use crate::encode::JobProgress;
use crate::encode::sample_entry::{
    build_mp4a_sample_entry, build_opus_sample_entry,
};
use crate::error::{Error, Result};

use super::{AudioOutput, EncodedAudioSample, RawSample, first_sample_entry, audio_params_of};
pub(crate) fn encode_audio(
    samples: &[RawSample],
    timescale: Option<NonZeroU32>,
    audio_codec: AudioCodec,
    bitrate_kbps: u32,
    progress: &JobProgress,
) -> Result<AudioOutput> {
    match audio_codec {
        AudioCodec::Aac => encode_audio_aac(samples, timescale, bitrate_kbps, progress),
        AudioCodec::Opus => encode_audio_opus(samples, timescale, bitrate_kbps, progress),
    }
}

/// 入力 SampleEntry から音声デコーダを選択して生成する
#[cfg(target_os = "macos")]
pub(crate) fn make_audio_decoder(entry: &SampleEntry) -> Result<AtDecoder> {
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
pub(crate) fn decode_audio_to_pcm(samples: &[RawSample]) -> Result<(Vec<i16>, u32, u8)> {
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
    progress: &JobProgress,
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
            composition_time_offset: None,
        });
        cumulative_samples += duration as u64;
        progress.add_processed(1);
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
    progress: &JobProgress,
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
            composition_time_offset: None,
        });
        cumulative_samples += duration as u64;
        progress.add_processed(1);
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
            composition_time_offset: None,
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
pub(crate) fn resample_linear(pcm: &[i16], from_hz: u32, to_hz: u32, channels: u8) -> Vec<i16> {
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
