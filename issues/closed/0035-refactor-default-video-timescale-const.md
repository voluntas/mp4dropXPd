# NonZeroU32::new(30).expect("30 != 0") の magic number 重複

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`src/encode/transcode.rs:361, 443, 552` で重複している `NonZeroU32::new(30).expect("30 != 0")` を `const` 化する。

## 優先度根拠

マジックナンバー 30 が 3 箇所で重複。意味 (デフォルトの映像 timescale = 30 fps) が不明確。

## 現状

`src/encode/transcode.rs:361, 443, 552`:

```rust
let ts = timescale.unwrap_or_else(|| NonZeroU32::new(30).expect("30 != 0"));
```

## 設計方針

`src/encode/transcode.rs` 冒頭に `const DEFAULT_VIDEO_TIMESCALE: NonZeroU32 = NonZeroU32::new(30).expect("30 != 0");` を定義し、3 箇所で使い回す。

## 完了条件

マジックナンバー 30 が 1 箇所に集約される。`cargo build` / `cargo test` が通る。

## 解決方法

1. `src/encode/transcode.rs` 冒頭の `use` 群の直後に `const DEFAULT_VIDEO_TIMESCALE: NonZeroU32 = NonZeroU32::new(30).expect("30 != 0");` を追加
2. 3 箇所の `NonZeroU32::new(30).expect("30 != 0")` を `DEFAULT_VIDEO_TIMESCALE` に置換

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0035 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0021 との関係**: 0021 (ファイル分割) と並行可 (定数定義は `transcode.rs` 冒頭で行う)。
- **0001-0008 との並行**: 本 issue は `transcode.rs` 冒頭の定数追加と 3 箇所の置換で、0001-0008 とは作業領域が重なる可能性がある (0001 が `transcode.rs:114-116`、0002 が `transcode.rs:515-540`、0003 が `transcode.rs:306-330`)。**0001-0008 → 0035 の順** でコミットする (0001-0008 の修正が安定してから 0035 で集約)。
- **0007 で整備される基盤**: 新規テスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] NonZeroU32::new(30) を DEFAULT_VIDEO_TIMESCALE const に集約` を 1 行で追記する。
