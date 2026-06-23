# panic 時の空 EncodeJob プレースホルダ削除

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`src/encode/mod.rs:90-100` の panic 時に空の `EncodeJob` を `Err` と組にして push する実装を削除する。

## 優先度根拠

呼び出し側 (app.rs:213-220) は `Result` しか使わないため、ジョブ内容は完全に無視。panic の握りつぶし実装。shiguredo-rust 規約「パニックは『絶対に発生しない想定が破られた = 実装バグ』の表明であり、握りつぶすべきではない」「通常のエラーは `Result` で表現する」に違反。

## 現状

`src/encode/mod.rs:90-100`:

```rust
Err(e) => out.push((
    EncodeJob::new(PathBuf::new(), PathBuf::new(), EncodeRecipe::default()),
    Err(Error::Message(format!("encode task failed: {e}"))),
)),
```

## 設計方針

panic 時は `?` で早期 return (または `expect("encode task panicked")`) して握りつぶさない。空の `EncodeJob` プレースホルダを削除する。

## 完了条件

panic 時に空の `EncodeJob` が作成されなくなる。`JoinError` は呼び出し元に伝播する。

## 解決方法

`src/encode/mod.rs:79-101` の `run_jobs_async` を以下のように修正する。

- `while let Some(joined) = set.join_next().await { match joined { Ok(pair) => out.push(pair), Err(e) => return Err(Error::Message(format!("encode task panicked: {e}"))), } }` に変更
- 空の `EncodeJob::new(PathBuf::new(), ...)` を削除

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0047 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0017 との関係**: 0017 (`bug-join-error-ui-disclosure`) は panic 時の UI 開示を扱う。0047 で panic を `?` で早期 return する場合、0017 側の UI 開示と組み合わせる必要あり。**0047 → 0017 の順** でコミットするか、**0047 と 0017 を統合** して 1 コミットにする。
- **0001-0008 との並行**: 本 issue は `src/encode/mod.rs:90-100` の修正で、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: `tests/test_encode.rs` への panic 伝播テスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を 1 箇所追加 (`"encode task panicked: {e}"`)。0018 着手時にこの 1 箇所が構造化バリアント (`Error::EncodePanic { source }` 等) へ置換される想定。
- **0032/0046 との関係**: panic 時のログを `tracing::error!` で残すかどうかは 0032 (`add-tracing-logging`) / 0046 (`refactor-remove-tracing-init`) の判断に従う。0047 では `tracing` 呼び出しを追加せず、0017 側で `tracing::error!` を入れる想定。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### 不具合修正` サブセクション (0009 で確立) に `[FIX] panic 時の空 EncodeJob プレースホルダを削除し JoinError を伝播` を 1 行で追記する。
