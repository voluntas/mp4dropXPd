# // finalize 英語コメントの日本語化 (#0016 と統合)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-finalize-comment-japanese
- Polished:
- Reporter:

## 目的

`src/encode/transcode.rs:1098` の `// finalize` 英語コメントを日本語化する。

## 優先度根拠

AGENTS.md「コメントは全て日本語にすること」違反。#0016 (英語コメント) と統合して対応。

## 現状

`src/encode/transcode.rs:1098`:

```rust
// finalize
let finalized = muxer
    .finalize()
    ...
```

## 設計方針

日本語コメントに書き換える。`#0016` の修正でまとめて対応。

## 完了条件

`src/encode/transcode.rs:1098` のコメントが日本語になる。

## 解決方法

#0016 を参照して対応。具体的には `// finalize` を `// muxer をファイナライズする` に置換。
