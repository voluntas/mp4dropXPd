# let _status = self.state.status.clone() のデッドコード削除

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-dead-status-clone
- Polished:
- Reporter:

## 目的

`src/app.rs:335` のデッドコードを削除する。

## 優先度根拠

`_status` は使われていない (Rust が `_` プレフィックスで警告を抑制)。clone 自体も無駄なアロケーション。

## 現状

`src/app.rs:335`:

```rust
let _status = self.state.status.clone();
```

## 設計方針

行ごと削除する。

## 完了条件

`src/app.rs:335` の行が削除される。`cargo build` / `cargo clippy` が警告なしで通る。

## 解決方法

`src/app.rs:335` の `let _status = self.state.status.clone();` を削除。
