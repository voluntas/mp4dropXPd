//! 出力 MP4 の SampleEntry 構築ヘルパー
use shiguredo_mp4::boxes::{
    AudioSampleEntryFields, Av01Box, Av1cBox, Avc1Box, AvccBox, DopsBox, EsdsBox, Hev1Box, HvccBox,
    HvccNalUintArray, Mp4aBox, OpusBox, SampleEntry, VisualSampleEntryFields,
};
use shiguredo_mp4::descriptors::{
    DecoderConfigDescriptor, DecoderSpecificInfo, EsDescriptor, SlConfigDescriptor,
};
use shiguredo_mp4::{FixedPointNumber, Uint};

/// H.264 (avc1) の SampleEntry を構築する
/// H.264 (avc1) の SampleEntry を構築する
///
/// 仕様参照: ISO/IEC 14496-15 § 8.3.2.1.2 (AVC/H.264 SampleEntry)
/// 仕様変更時は本関数の実装を追従させる必要がある
pub fn build_avc1_sample_entry(sps: &[u8], pps: &[u8], width: u16, height: u16) -> SampleEntry {
    let avc_profile_indication = sps.first().copied().unwrap_or(66);
    let profile_compatibility = sps.get(1).copied().unwrap_or(0);
    let avc_level_indication = sps.get(2).copied().unwrap_or(30);

    // profile_indication が 66/77/88 以外 (High 等) の場合は chroma_format 等が必須
    let needs_ext = !matches!(avc_profile_indication, 66 | 77 | 88);

    SampleEntry::Avc1(Avc1Box {
        visual: visual_fields(width, height),
        avcc_box: AvccBox {
            avc_profile_indication,
            profile_compatibility,
            avc_level_indication,
            length_size_minus_one: Uint::new(3),
            sps_list: vec![sps.to_vec()],
            pps_list: vec![pps.to_vec()],
            chroma_format: needs_ext.then(|| Uint::new(1)), // クロマフォーマット 4:2:0
            bit_depth_luma_minus8: needs_ext.then(|| Uint::new(0)), // 8 ビット
            bit_depth_chroma_minus8: needs_ext.then(|| Uint::new(0)), // 8 ビット
            sps_ext_list: Vec::new(),
        },
        unknown_boxes: Vec::new(),
    })
}

/// H.265 (hev1) の SampleEntry を構築する
/// HEVC/H.265 (hev1) の SampleEntry を構築する
///
/// 仕様参照: ISO/IEC 14496-15 § 8.3.2.1.3 (HEVC/H.265 SampleEntry)
/// 仕様変更時は本関数の実装を追従させる必要がある
pub fn build_hev1_sample_entry(
    vps: &[u8],
    sps: &[u8],
    pps: &[u8],
    width: u16,
    height: u16,
) -> SampleEntry {
    // VPS=32、SPS=33、PPS=34 (HEVC NAL ユニットタイプ)
    let nalu_arrays = vec![
        HvccNalUintArray {
            array_completeness: Uint::new(0),
            nal_unit_type: Uint::new(32),
            nalus: vec![vps.to_vec()],
        },
        HvccNalUintArray {
            array_completeness: Uint::new(0),
            nal_unit_type: Uint::new(33),
            nalus: vec![sps.to_vec()],
        },
        HvccNalUintArray {
            array_completeness: Uint::new(0),
            nal_unit_type: Uint::new(34),
            nalus: vec![pps.to_vec()],
        },
    ];

    SampleEntry::Hev1(Hev1Box {
        visual: visual_fields(width, height),
        hvcc_box: HvccBox {
            general_profile_space: Uint::new(0),
            general_tier_flag: Uint::new(0),
            general_profile_idc: Uint::new(1),     // Main プロファイル
            general_profile_compatibility_flags: 0x60000000,
            general_constraint_indicator_flags: Uint::new(0),
            general_level_idc: 0,
            min_spatial_segmentation_idc: Uint::new(0),
            parallelism_type: Uint::new(0),
            chroma_format_idc: Uint::new(1), // クロマフォーマット 4:2:0
            bit_depth_luma_minus8: Uint::new(0),
            bit_depth_chroma_minus8: Uint::new(0),
            avg_frame_rate: 0,
            constant_frame_rate: Uint::new(0),
            num_temporal_layers: Uint::new(1),
            temporal_id_nested: Uint::new(1),
            length_size_minus_one: Uint::new(3),
            nalu_arrays,
        },
        unknown_boxes: Vec::new(),
    })
}

/// AV1 (av01) の SampleEntry を構築する
/// AV1 (av01) の SampleEntry を構築する
///
/// 仕様参照: ISO/IEC 14496-15 § 8.3.2.1.4 (AV1 SampleEntry)
/// 仕様変更時は本関数の実装を追従させる必要がある
pub fn build_av01_sample_entry(extra_data: &[u8], width: u16, height: u16) -> SampleEntry {
    // SVT-AV1 の extra_data は OBU シーケンスヘッダーそのもの
    // av1C box の config_obus にそのまま格納する
    // seq_profile / seq_level_idx_0 等は extra_data からパースするのが本来だが、
    // ここでは簡単のためデフォルト値 (profile=0, level=0) を使う
    SampleEntry::Av01(Av01Box {
        visual: visual_fields(width, height),
        av1c_box: Av1cBox {
            seq_profile: Uint::new(0),
            seq_level_idx_0: Uint::new(0),
            seq_tier_0: Uint::new(0),
            high_bitdepth: Uint::new(0),
            twelve_bit: Uint::new(0),
            monochrome: Uint::new(0),
            chroma_subsampling_x: Uint::new(1), // クロマフォーマット 4:2:0
            chroma_subsampling_y: Uint::new(1),
            chroma_sample_position: Uint::new(0),
            initial_presentation_delay_minus_one: None,
            config_obus: extra_data.to_vec(),
        },
        unknown_boxes: Vec::new(),
    })
}

/// AAC (mp4a) の SampleEntry を構築する
/// AAC (mp4a) の SampleEntry を構築する
///
/// 仕様参照: ISO/IEC 14496-12 § 12.2.2.2 + ISO/IEC 14496-3 (AAC)
/// 仕様変更時は本関数の実装を追従させる必要がある
pub fn build_mp4a_sample_entry(sample_rate: u32, channels: u8) -> SampleEntry {
    let asc = build_aac_audio_specific_config(sample_rate, channels);

    SampleEntry::Mp4a(Mp4aBox {
        audio: audio_fields(sample_rate, channels),
        esds_box: EsdsBox {
            es: EsDescriptor {
                es_id: 1,
                stream_priority: EsDescriptor::LOWEST_STREAM_PRIORITY,
                depends_on_es_id: None,
                url_string: None,
                ocr_es_id: None,
                dec_config_descr: DecoderConfigDescriptor {
                    object_type_indication:
                        DecoderConfigDescriptor::OBJECT_TYPE_INDICATION_AUDIO_ISO_IEC_14496_3,
                    stream_type: DecoderConfigDescriptor::STREAM_TYPE_AUDIO,
                    up_stream: DecoderConfigDescriptor::UP_STREAM_FALSE,
                    buffer_size_db: Uint::new(0),
                    max_bitrate: 0,
                    avg_bitrate: 0,
                    dec_specific_info: Some(DecoderSpecificInfo { payload: asc }),
                },
                sl_config_descr: SlConfigDescriptor,
            },
        },
        unknown_boxes: Vec::new(),
    })
}

/// Opus (Opus) の SampleEntry を構築する
/// Opus の SampleEntry を構築する
///
/// 仕様参照: RFC 7845 § 5 (OpusSampleEntry)
/// 仕様変更時は本関数の実装を追従させる必要がある
pub fn build_opus_sample_entry(sample_rate: u32, channels: u8, pre_skip: u16) -> SampleEntry {
    SampleEntry::Opus(OpusBox {
        audio: audio_fields(sample_rate, channels),
        dops_box: DopsBox {
            output_channel_count: channels,
            pre_skip,
            input_sample_rate: sample_rate,
            output_gain: 0,
        },
        unknown_boxes: Vec::new(),
    })
}

fn visual_fields(width: u16, height: u16) -> VisualSampleEntryFields {
    VisualSampleEntryFields {
        data_reference_index: VisualSampleEntryFields::DEFAULT_DATA_REFERENCE_INDEX,
        width,
        height,
        horizresolution: VisualSampleEntryFields::DEFAULT_HORIZRESOLUTION,
        vertresolution: VisualSampleEntryFields::DEFAULT_VERTRESOLUTION,
        frame_count: VisualSampleEntryFields::DEFAULT_FRAME_COUNT,
        compressorname: VisualSampleEntryFields::NULL_COMPRESSORNAME,
        depth: VisualSampleEntryFields::DEFAULT_DEPTH,
    }
}

fn audio_fields(sample_rate: u32, channels: u8) -> AudioSampleEntryFields {
    AudioSampleEntryFields {
        data_reference_index: AudioSampleEntryFields::DEFAULT_DATA_REFERENCE_INDEX,
        channelcount: channels as u16,
        samplesize: AudioSampleEntryFields::DEFAULT_SAMPLESIZE,
        samplerate: FixedPointNumber::new(sample_rate as u16, 0),
    }
}

/// ISO/IEC 14496-3 AudioSpecificConfig (AAC-LC, no SBR) を構築する
///
/// `samplingFrequencyIndex` が 0xf (escape) の場合は 24-bit のサンプリングレートを埋め込む
/// AAC AudioSpecificConfig を構築する
///
/// 仕様参照: ISO/IEC 14496-3 § 1.6.2.1 (AudioSpecificConfig)
/// 仕様変更時は本関数の実装を追従させる必要がある
fn build_aac_audio_specific_config(sample_rate: u32, channels: u8) -> Vec<u8> {
    const AUDIO_OBJECT_TYPE_AAC_LC: u8 = 2;
    let freq_index = sampling_frequency_index(sample_rate);

    if freq_index < 0xf {
        let byte1 = (AUDIO_OBJECT_TYPE_AAC_LC << 3) | (freq_index >> 1);
        let byte2 = ((freq_index & 1) << 7) | (channels << 3);
        vec![byte1, byte2]
    } else {
        let byte1 = (AUDIO_OBJECT_TYPE_AAC_LC << 3) | 0b1111;
        vec![
            byte1,
            (sample_rate >> 16) as u8,
            (sample_rate >> 8) as u8,
            sample_rate as u8,
            channels << 3,
        ]
    }
}

fn sampling_frequency_index(sample_rate: u32) -> u8 {
    match sample_rate {
        96000 => 0,
        88200 => 1,
        64000 => 2,
        48000 => 3,
        44100 => 4,
        32000 => 5,
        24000 => 6,
        22050 => 7,
        16000 => 8,
        12000 => 9,
        11025 => 10,
        8000 => 11,
        7350 => 12,
        _ => 0xf,
    }
}
