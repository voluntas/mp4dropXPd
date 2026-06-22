# tracing / tracing-subscriber 依存削除 (#0032 と統合)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-tracing-init
- Polished:
- Reporter:

## 目的

`src/main.rs:11-15` の `tracing_subscriber::fmt()...init()` と `Cargo.toml:31, 33` の `tracing` / `tracing-subscriber` 依存を削除する。

## 優先度根拠

`tracing::*!` マクロの呼び出しが 0 件。AGENTS.md「Premature Optimization is the Root of All Evil」と shiguredo-rust 規約 (暗黙的に YAGNI) に反する。デッドインフラ。

## 現状

`src/main.rs:11-15`:

```rust
tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

`Cargo.toml:31, 33`:

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

## 設計方針

#0032 (ログ出力の追加) と統合して判断する。

- #0032 で `tracing::error!` 等を追加する場合 → この issue は close (S-9 撤回)
- #0032 を採用しない場合 → この issue で `tracing` 依存を削除

## 完了条件

#0032 で `tracing` を使う場合 → この issue は close。

#0032 で `tracing` を使わない場合:

1. `Cargo.toml:31` の `tracing = "0.1"` を削除
2. `Cargo.toml:33` の `tracing-subscriber = ...` を削除
3. `src/main.rs:11-15` の `tracing_subscriber::fmt()...init();` ブロックを削除

## 解決方法

#0032 の判断に従う。#0032 を採用するなら close。採用しないなら依存と init を削除。
