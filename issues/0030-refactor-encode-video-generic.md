# H.264 / H.265 / AV1 エンコード関数の構造的重複

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-encode-video-generic
- Polished:
- Reporter:

## 目的

`encode_video_h264` / `encode_video_h265` / `encode_video_av1` の 3 関数を共通ヘルパに抽出する。

## 優先度根拠

3 関数が 80% 同じ骨格 (デコードループ・進捗加算・flush・SPS/PPS 抽出・SampleEntry 構築)。H.264 と H.265 は 90% 共通。保守性とバグ混入リスクの双方で問題。

## 現状

`src/encode/transcode.rs:352-513` (h264 + h265) と `src/encode/transcode.rs:542-638` (av1) で 3 関数が類似した構造を持つ。

## 設計方針

`encode_video_generic(encoder_ctor, nal_extractor, sample_entry_builder)` 形に高階関数で抽象化し、差分 (エンコーダ生成、NAL パラメータ抽出、SampleEntry 構築) のみを引数で渡す。

## 完了条件

3 関数の重複コードが大幅に削減される。バグ修正時に 1 箇所の修正で済む。

## 解決方法

1. `src/encode/transcode/video.rs` (0021 のリファクタで分離後) に `encode_video_generic<F, G, H, E>(samples, timescale, bitrate_kbps, make_encoder, extract_nal, make_sample_entry, progress) -> Result<VideoOutput>` を導入
2. `encode_video_h264` / `encode_video_h265` / `encode_video_av1` は差分のクロージャを引数で渡すだけにする
3. AV1 の SVT-AV1 特有パスは別系統として残す
