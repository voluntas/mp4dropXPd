# app.rs 1222 行 / transcode.rs 1121 行の肥大化

- Priority: Medium
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

`app.rs` (1,222 行) と `transcode.rs` (1,121 行) を機能別モジュールに分割する。

## 優先度根拠

単一ファイルに 1000 行超のコード。3 ウィンドウ + 状態管理 + UI ヘルパーが app.rs に、デコード + 3 映像エンコード + 2 音声エンコード + mux + ヘルパーが transcode.rs に同居。保守性低下とレビュー単位の肥大化を招く。

## 現状

- `src/app.rs` (1,222 行): `SharedState` / `SharedGlobal`, `DropState`, `DropWindow`, `MenuWindow`, `OptionsWindow`, ビットレート定数、ヘルパー
- `src/encode/transcode.rs` (1,121 行): demux / decode / 3 映像エンコード (H.264/H.265/AV1) / 2 音声エンコード (AAC/Opus) / mux / ヘルパー / cfg スタブ

## 設計方針

機能別に分割する。

- `src/app/{mod, state, drop_window, menu_window, options_window, ui_helpers}.rs`
- `src/encode/transcode/{mod, video, audio, mux, params}.rs`

## 完了条件

`app.rs` と `transcode.rs` の行数がそれぞれ 200 行以下になる。機能別モジュールに分割される。

## 解決方法

1. `src/app/mod.rs` を作成し、サブモジュールを `mod` で宣言
2. `src/app/state.rs` に `SharedState`, `SharedGlobal`, `AppState` を移動
3. `src/app/drop_window.rs` に `DropState`, `DropWindow` を移動
4. `src/app/menu_window.rs` に `MenuWindow` を移動
5. `src/app/options_window.rs` に `OptionsWindow` とビットレート定数を移動
6. `src/app/ui_helpers.rs` に `menu_sep`, `menu_label`, `options_section_label`, `options_row`, `bitrate_label` を移動
7. `src/encode/transcode/mod.rs` を作成し、サブモジュールを `mod` で宣言
8. `src/encode/transcode/video.rs` に `decode_video_frames`, `make_vt_decoder`, `encode_video`, `encode_video_h264`, `encode_video_h265`, `encode_video_av1`, `build_video_samples`, `drain_vt_encoder` を移動
9. `src/encode/transcode/audio.rs` に `encode_audio`, `make_audio_decoder`, `decode_audio_to_pcm`, `encode_audio_aac`, `encode_audio_opus`, `resample_linear` を移動
10. `src/encode/transcode/mux.rs` に `write_mp4` を移動
11. `src/encode/transcode/params.rs` に `first_sample_entry`, `resolution_of`, `audio_params_of`, `extract_h265_params`, `copy_stride`, `average_duration`, `average_bitrate_kbps` を移動

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0001-0008 すべて完了 → 0021 (本 issue)** の順で develop に直接コミットする。本 issue は大規模リファクタで、0001-0008 の修正が安定した後に着手する。
- **0001-0008 との関係**: 0001-0008 の修正対象 (`transcode.rs:114-116`, `515-540`, `306-330`, `986-1109`, `174-205`, `103-106`, `src/app.rs` UI 文字列) は本 issue の分割後の各ファイルに散らばる。**0001-0008 → 0021 の順** でコミットする。
- **0019 / 0020 との関係**: 0019 (`SharedState` の Entity 化) と 0020 (`pending_*` フラグ enum 化) は `src/app.rs` の構造を変える。本 issue と 0019/0020 の両方が `src/app.rs` を変更するため、**0019/0020 → 0021 の順** でコミットするか、**0019/0020 と 0021 を統合** して 1 コミットにする。
- **0029 / 0030 との関係**: 0029 (`refactor-progress-to-event-driven`) と 0030 (`refactor-encode-video-generic`) は `transcode.rs` の構造を変える。これも本 issue と並行不可。**0029/0030 → 0021 の順** でコミットするか、**0021 完了後** に 0029/0030 に着手。
- **0007 で整備される基盤**: 本 issue 完了後の各モジュールに対して個別テストを追加するのは 0007 の基盤に依存。0007 完了後に別 issue で対応。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
