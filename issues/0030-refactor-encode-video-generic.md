# H.264 / H.265 / AV1 エンコード関数の構造的重複

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0001-0008 すべて完了 → 0021 → 0030 (本 issue)** の順で develop に直接コミットする。0021 (ファイル分割) 完了後に着手。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0001, 0002, 0003 との関係**: 0001 (破損 MP4 slice), 0002 (drain timeout), 0003 (dav1d error) は `transcode.rs:114-116, 515-540, 306-330` を変更する。本 issue は `transcode.rs:352-513, 542-638` のリファクタで、0001-0003 完了後に着手。
- **0007 で整備される基盤**: ドキュメント/リファクタのみなので新規テスト不要だが、0030 完了後の各エンコーダ関数のテストは 0007 基盤利用。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] encode_video_h264/h265/av1 の重複を encode_video_generic に集約` を 1 行で追記する。
