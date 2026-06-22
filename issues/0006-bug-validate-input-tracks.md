# 入力 MP4 のトラック検証不足 (映像も音声もない MP4 で破損ファイル生成)

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-validate-input-tracks
- Polished:
- Reporter:

## 目的

映像トラックも音声トラックも含まない MP4 を入力したときに、破損ファイルを生成せず早期にエラーを返す。

## 優先度根拠

入力検証の欠落。破損ファイルの生成は UI/ユーザー双方に無用な混乱を与える。

## 現状

`src/encode/transcode.rs:106` で `video_idx` と `audio_idx` を `tracks.iter().position(...)` で取得するが、両方とも `None` のケースは検出されず `write_mp4` まで到達する。

## 設計方針

`tracks` のループ直後に「映像も音声もない」状態を検出して `Error::Message("no video/audio track")` を返す。

## 完了条件

映像も音声も持たない MP4 を入力すると、即座に `Err` が返り、破損ファイルが生成されない。

## 解決方法

`src/encode/transcode.rs:103-106` 直後に以下を追加する。

```rust
if tracks.iter().all(|t| t.kind != TrackKind::Video && t.kind != TrackKind::Audio) {
    return Err(Error::Message("no video/audio track in input".into()));
}
```

加えて `tests/test_encode.rs` で空トラックのみの MP4 入力のテストを追加する。
