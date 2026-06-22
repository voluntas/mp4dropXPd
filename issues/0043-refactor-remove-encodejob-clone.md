# EncodeJob の不要 Clone derive 削除

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-encodejob-clone
- Polished:
- Reporter:

## 目的

`src/encode/job.rs:5` の `#[derive(Clone)]` を削除する。

## 優先度根拠

`EncodeJob::clone()` 呼び出しが 0 件。`input.clone()` (app.rs:170) は `PathBuf` のクローン。所有権ごと spawn に move される。

## 現状

`src/encode/job.rs:5`:

```rust
#[derive(Debug, Clone)]
pub struct EncodeJob {
    ...
}
```

## 設計方針

`#[derive(Debug, Clone)]` → `#[derive(Debug)]` に削減。

## 完了条件

`#[derive(Clone)]` が削除される。`cargo build` / `cargo clippy` が警告なしで通る。

## 解決方法

`src/encode/job.rs:5` の `#[derive(Debug, Clone)]` を `#[derive(Debug)]` に変更。
