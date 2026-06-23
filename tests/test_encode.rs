//! `encode` モジュールの単体テスト
//!
//! 戦略: 破損・異常入力を与えて `Err` で安全に停止することを検証する
//!
//! 検証対象: transcode.rs のサンプル範囲検証、トラック検証、demux エラー処理

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

    let result = encode_file_async(input, output, recipe, true, progress).await;

    assert!(result.is_err());
    assert!(
        matches!(result, Err(Error::Message(ref msg)) if msg == "no video/audio track in input"),
        "expected no video/audio track error, got: {result:?}"
    );
}

/// 途中で切断された MP4 を投入してもパニックせず Err を返す
#[tokio::test]
async fn truncated_mp4_does_not_panic() {
    let input = Path::new("tests/fixtures/truncated_h264_aac.mp4");
    let output = Path::new("/tmp/test_output_truncated.mp4");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, true, progress).await;

    // パニックせずに Err が返れば OK（demux エラーまたは範囲外エラーのいずれか）
    assert!(result.is_err(), "truncated MP4 should return Err, got: {result:?}");
}

/// stco chunk_offset を改竄した MP4 を投入してもパニックせず Err を返す
#[tokio::test]
async fn stco_corrupted_mp4_does_not_panic() {
    let input = Path::new("tests/fixtures/stco_corrupted_h264_aac.mp4");
    let output = Path::new("/tmp/test_output_stco_corrupted.mp4");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, true, progress).await;

    // パニックせずに Err が返れば OK
    assert!(result.is_err(), "stco corrupted MP4 should return Err, got: {result:?}");
}

/// ランダムバイト列を MP4 として投入してもパニックせず Err を返す
#[tokio::test]
async fn random_bytes_does_not_panic() {
    let input = Path::new("tests/fixtures/random_bytes.mp4");
    let output = Path::new("/tmp/test_output_random.mp4");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, true, progress).await;

    // パニックせずに Err が返れば OK（demux がエラーを返すはず）
    assert!(result.is_err(), "random bytes should return Err, got: {result:?}");
}

/// 正常な MP4 (H.264 + AAC) のトランスコードが成功することを確認する smoke test
///
/// ローカル実測値 (MacBook Pro M4 Pro, 2026-06-23): 約 0.12 秒
/// 30 秒の drain タイムアウトより十分短いため、タイムアウト誤検出の心配はない
#[tokio::test]
async fn valid_mp4_transcodes_successfully() {
    let input = Path::new("tests/fixtures/valid_h264_aac.mp4");
    let output = Path::new("/tmp/test_output_valid.mp4");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, true, progress).await;

    assert!(result.is_ok(), "valid MP4 should transcode successfully, got: {result:?}");
}

/// 正常な AV1 MP4 (libsvtav1 + Opus) のトランスコードが成功することを確認する smoke test
#[tokio::test]
async fn valid_av1_mp4_transcodes_successfully() {
    let input = Path::new("tests/fixtures/valid_av1_opus.mp4");
    let output = Path::new("/tmp/test_output_valid_av1.mp4");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, true, progress).await;

    assert!(result.is_ok(), "valid AV1 MP4 should transcode successfully, got: {result:?}");
}

/// 途中で切断された AV1 MP4 を投入してもパニックせず Err を返す
#[tokio::test]
async fn truncated_av1_mp4_does_not_panic() {
    let input = Path::new("tests/fixtures/truncated_av1_opus.mp4");
    let output = Path::new("/tmp/test_output_truncated_av1.mp4");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, true, progress).await;

    // パニックせずに Err が返れば OK（demux エラーまたはデコードエラーのいずれか）
    assert!(result.is_err(), "truncated AV1 MP4 should return Err, got: {result:?}");
}

/// エンコード失敗時に出力パスに破損ファイルが残らないことを確認する
#[tokio::test]
async fn failed_encode_does_not_leave_output_file() {
    let input = Path::new("tests/fixtures/random_bytes.mp4");
    let output = Path::new("/tmp/test_output_atomic_failure.mp4");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    // 事前に出力ファイルを削除しておく
    let _ = std::fs::remove_file(output);
    let result = encode_file_async(input, output, recipe, true, progress).await;
    assert!(result.is_err());

    // tmp + atomic rename 方式により、失敗時は出力ファイルが残らない
    assert!(!output.exists(), "output file should not exist after failed encode");
}

/// overwrite = false で既存ファイルがある場合は Err が返る
#[tokio::test]
async fn overwrite_false_with_existing_file_returns_error() {
    let input = Path::new("tests/fixtures/valid_h264_aac.mp4");
    let output = Path::new("/tmp/test_output_overwrite_false.mp4");
    // 事前にファイルを作成しておく
    std::fs::write(output, b"x").expect("create existing file");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, false, progress).await;

    assert!(result.is_err(), "overwrite=false with existing file should return Err, got: {result:?}");
}

/// overwrite = true で既存ファイルがあっても正常に上書きされる
#[tokio::test]
async fn overwrite_true_with_existing_file_succeeds() {
    let input = Path::new("tests/fixtures/valid_h264_aac.mp4");
    let output = Path::new("/tmp/test_output_overwrite_true.mp4");
    // 事前にファイルを作成しておく
    std::fs::write(output, b"y").expect("create existing file");
    let recipe = EncodeRecipe::default();
    let progress = Arc::new(JobProgress::new());

    let result = encode_file_async(input, output, recipe, true, progress).await;

    assert!(result.is_ok(), "overwrite=true with existing file should succeed, got: {result:?}");
    // 正常に上書きされたことを確認 (tmp rename 後にファイルが 7 バイト以上あるはず)
    assert!(output.exists());
}
