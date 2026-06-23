# tracing / tracing-subscriber 依存削除 (#0032 と統合)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0032 → 0046 (本 issue)** の順で develop に直接コミットする。0032 (`tracing` 採用判断) の後に着手。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0032 との関係**: 0032 で `tracing` 採用なら本 issue は close。0032 で採用しないなら本 issue で `tracing` 依存と init を削除する。**0032 の判断を待つ**。
- **0001-0008 との関係**: 0001, 0002, 0003 で `tracing::warn!` が追加されている。0032 で `tracing` 採用なら本 issue は close、0001-0008 のログ呼び出しが機能する。0032 で採用しないなら 0001-0008 の `tracing::warn!` 呼び出しもデッドコード化するため、0001-0008 を re-polish してログ呼び出しを削除する必要がある。
- **0007 で整備される基盤**: `tracing-test` 等のテストユーティリティ追加は 0032 と統合して判断。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 0032 で採用するなら影響なし、0032 で採用しないなら `tracing` 依存削除後に `cargo build` が通ることを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] tracing / tracing-subscriber 依存削除 (0032 で採用しない場合)` を 1 行で追記する (0032 で採用する場合は追記なし、close)。
