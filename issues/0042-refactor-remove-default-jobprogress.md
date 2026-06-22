# impl Default for JobProgress の未使用 impl 削除

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-default-jobprogress
- Polished:
- Reporter:

## 目的

`src/encode/mod.rs:29-33` の `impl Default for JobProgress` を削除する。

## 優先度根拠

`JobProgress::default()` の使用箇所が 0 件。`JobProgress::new()` 経由でのみ生成されている。`Default` 実装は未使用のデッドコード。

## 現状

`src/encode/mod.rs:29-33`:

```rust
impl Default for JobProgress {
    fn default() -> Self {
        Self::new()
    }
}
```

## 設計方針

`impl Default for JobProgress` ブロックを削除する。

## 完了条件

`impl Default for JobProgress` が削除される。`cargo build` / `cargo clippy` が警告なしで通る。

## 解決方法

`src/encode/mod.rs:29-33` の `impl Default for JobProgress { fn default() -> Self { Self::new() } }` ブロックを削除。
