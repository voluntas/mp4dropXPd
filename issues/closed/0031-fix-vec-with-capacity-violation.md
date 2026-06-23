# Vec::with_capacity 規約違反 (transcode.rs:872)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`resample_linear` 内の `Vec::with_capacity` を `Vec::new()` に置換する。

## 優先度根拠

shiguredo-rust 規約「入力バイナリデータをデコードする際には `Vec::with_capacity()` などのメモリを事前に割り当てるメソッドを原則として使用しないこと」に違反。

## 現状

`src/encode/transcode.rs:872`:

```rust
let mut out = Vec::with_capacity(output_frames as usize * channels);
```

`output_frames` は入力 PCM (`pcm.len()`) から派生するため、攻撃的に大きな to_hz で過剰確保になる可能性。

## 設計方針

`Vec::new()` に置換。性能差は `push` の reallocate 数回分のみで実用上無視できる。

## 完了条件

`Vec::with_capacity` の使用が消える。`cargo test` が通る (機能変化なし)。

## 解決方法

`src/encode/transcode.rs:872` の `Vec::with_capacity(output_frames as usize * channels)` を `Vec::new()` に置換。

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0031 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0001-0008 との並行**: 本 issue は `transcode.rs:872` の 1 行修正で、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: テストは 0001-0006 の `tests/test_encode.rs` で resample_linear の動作確認に統合可能。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[FIX] resample_linear の Vec::with_capacity を Vec::new() に置換 (shiguredo-rust 規約違反修正)` を 1 行で追記する。
