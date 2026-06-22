# NonZeroU32::new(30).expect("30 != 0") の magic number 重複

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-default-video-timescale-const
- Polished:
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
