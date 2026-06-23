//! `encode` モジュールの単体テスト
//!
//! 戦略: 破損・異常入力を与えて `Err` で安全に停止することを検証する

use std::path::Path;
use std::sync::Arc;

use mp4dropxpd::codec::EncodeRecipe;
use mp4dropxpd::encode::{JobProgress, encode_file_async};
use mp4dropxpd::error::Error;

/// 字幕のみの MP4 は映像/音声トラックが存在しないため Err が返る
#[tokio::test]
async fn subtitle_only_mp4_returns_error() {
    let input = Path::new("tests/fixtures/subtitle_only.mp4");
    let output = Path::new("/tmp/test_output_subtitle_only.mp4");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, progress).await;

    assert!(result.is_err());
    assert!(
        matches!(result, Err(Error::Message(ref msg)) if msg == "no video/audio track in input"),
        "expected no video/audio track error, got: {result:?}"
    );
}
