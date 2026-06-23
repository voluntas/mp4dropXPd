# Error::Message(String) 依存で構造化エラー型が未定義

- Priority: Medium
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0001-0008 すべて完了 → 0018 (本 issue)** の順で develop に直接コミットする。本 issue はリファクタで、0001-0008 が追加した `Error::Message` 箇所を構造化バリアントに置換するため、0001-0008 の完了が前提。
- **0001, 0002, 0003, 0004, 0005, 0006 との関係**: これらの issue で追加した `Error::Message` 3 箇所 (0001), 1 箇所 (0002), 1 箇所 (0003) 等を本 issue で構造化バリアントに置換する。**0001-0008 → 0018 の順** でコミットする。
- **0007 で整備される基盤**: `tests/test_error.rs` への単体テスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 着手時の具体的置換対象**:
  - 0001: `Error::InvalidSampleRange { start, end, file_size }` バリアント新設
  - 0002: `Error::DrainTimeout { elapsed, expected, received }` バリアント新設 (0018:27 で言及済み)
  - 0003: `Error::Dav1dDecode { phase: &'static str }` または `Error::BitDepth { expected, actual }` バリアント新設
  - 0004: `Error::Rename { from, to }` バリアント新設
  - 0005: `Error::TrackEncode { track: TrackKind, message }` バリアント新設
  - 0006: `Error::NoTrack { kind: Option<TrackKind> }` バリアント新設
- **0018 着手前のバリアント名確定**: 上記バリアント名は本 issue 内で確定し、0001-0008 の `Error::Message` 追加箇所を本 issue で順次置換する。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
