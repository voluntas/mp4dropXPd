# Error::Message(String) 依存で構造化エラー型が未定義

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-error-message-to-structured
- Polished:
- Reporter:

## 目的

`Error::Message(String)` の濫用 (約 47 箇所) をやめ、構造化エラー型に置き換える。

## 優先度根拠

`From<shiguredo_*::Error>` 実装が error.rs:48-78 にあるのに `map_err(|e| Error::Message(format!("...: {e}")))` で文字列化。エラー型・文脈情報が失われ、UI 側で種別判別・リトライ・ユーザー通知ができない。

## 現状

- `src/error.rs:6` で `Error::Message(String)` 定義
- `From<DemuxError>` 実装 (error.rs:80-84) も結局 `Self::Message(format!("mp4 demux error: {e}"))` に変換
- `src/encode/transcode.rs` で約 47 箇所の `map_err(|e| Error::Message(format!(...)))`

## 設計方針

専用バリアント (`Error::Demux(DemuxError)`, `Error::Decode { codec, message }`, `Error::UnsupportedCodec { codec }`, `Error::DrainTimeout`, `Error::Mux { source }` 等) を追加し、ライブラリ由来エラーは対応する `From` 実装で変換する。`Error::Message(String)` は汎用フォールバック用に維持。

## 完了条件

`Error::Message(String)` の使用箇所が約 47 箇所から 10 箇所以下に削減される。`Error::Message` を使用する場合は仕様由来 (例: 仕様文から決定される文字列) に限定。

## 解決方法

1. `src/error.rs:4-17` の `Error` enum に `Demux(DemuxError)`, `Decode { codec: &'static str, message: String }`, `Mux { source: Box<dyn std::error::Error + Send + Sync> }` 等のバリアントを追加
2. `error.rs:80-84` の `From<DemuxError>` 実装を `Self::Demux(e)` に変更
3. `src/encode/transcode.rs` の `map_err(|e| Error::Message(format!("xxx error: {e}")))` を `?` または `.map_err(Into::into)?` に置換
4. `tests/test_error.rs` で Display の各バリアントを検証
