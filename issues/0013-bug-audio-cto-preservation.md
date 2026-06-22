# 入力音声サンプルの composition_time_offset が mux 時に破棄される

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-audio-cto-preservation
- Polished:
- Reporter:

## 目的

入力音声サンプルが `composition_time_offset` (CTO) を持っている場合に、mux 時に保持して映像と音声のタイミングを一致させる。

## 優先度根拠

音声と映像の再生タイミングが 1 サンプル単位でずれる。AAC 等の非典型例で問題。

## 現状

`src/encode/transcode.rs:1086` (音声 mux):

```rust
composition_time_offset: None,  // ←入力 CTO を握り潰している
```

`EncodedAudioSample` には `composition_time_offset` フィールドがない。

## 設計方針

`EncodedAudioSample` に `composition_time_offset: Option<i64>` を追加し、入力 CTO を保持して mux に渡す。

## 完了条件

CTO を持つ音声入力で、映像と音声の再生タイミングが一致する (リップシンクの破綻なし)。

## 解決方法

1. `src/encode/transcode.rs:60-64` の `EncodedAudioSample` に `pub composition_time_offset: Option<i64>` フィールドを追加
2. `src/encode/transcode.rs:761-766, 822-828, 838-844` の各音声サンプル生成で `RawSample::composition_time_offset` をコピー
3. `src/encode/transcode.rs:1071-1094` の音声 mux で `composition_time_offset: s.composition_time_offset` に変更
