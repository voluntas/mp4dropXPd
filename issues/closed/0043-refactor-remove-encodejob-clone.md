# EncodeJob の不要 Clone derive 削除

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0043 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0001-0008 との並行**: 本 issue は `src/encode/job.rs:5` の derive 削減で、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: 新規テスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] EncodeJob の未使用 Clone derive を削除` を 1 行で追記する。
