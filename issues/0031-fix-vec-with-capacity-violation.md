# Vec::with_capacity 規約違反 (transcode.rs:872)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-vec-with-capacity-violation
- Polished:
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
