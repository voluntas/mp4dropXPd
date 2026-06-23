//! `AudioCodec::label()` / `VideoCodec::label()` の固定値検証
//!
//! 戦略: label() は技術用語例外として英語のまま維持されることを assert する

use mp4dropxpd::codec::{AudioCodec, VideoCodec};

/// `AudioCodec::label()` が技術用語例外として英語を返すことを確認する
#[test]
fn audio_codec_labels_are_technical_terms() {
    assert_eq!(AudioCodec::Aac.label(), "AAC");
    assert_eq!(AudioCodec::Opus.label(), "Opus");
}

/// `VideoCodec::label()` が技術用語例外として英語を返すことを確認する
#[test]
fn video_codec_labels_are_technical_terms() {
    assert_eq!(VideoCodec::H264.label(), "H.264");
    assert_eq!(VideoCodec::H265.label(), "H.265");
    assert_eq!(VideoCodec::Av1.label(), "AV1");
}
