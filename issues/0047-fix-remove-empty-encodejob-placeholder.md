# panic 時の空 EncodeJob プレースホルダ削除

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-remove-empty-encodejob-placeholder
- Polished:
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
