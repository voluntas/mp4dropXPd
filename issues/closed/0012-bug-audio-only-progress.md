# 音声のみ MP4 で進捗バーが 0% 固定

- Priority: Medium
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

映像を含まない MP4 (音声のみ) を入力したときに、進捗バーが 0% のまま固まる問題を解消する。

## 優先度根拠

ユーザー体感を「ハング」と誤認させる UX 上の問題。音声のみ MP4 は実利用で普通にありえる。

## 現状

`src/encode/transcode.rs:154`:

```rust
progress.set_total(video_samples.len() as u64);
```

映像なし MP4 では `total = 0` → `JobProgress::ratio()` は常に 0.0。音声エンコーダ (`encode_audio_aac` / `encode_audio_opus`) には `progress` パラメータが渡されておらず、`add_processed(1)` も呼ばれない (transcode.rs:194-200)。

## 設計方針

音声エンコーダにも `progress: &JobProgress` を渡し、PCM フレーム単位で `add_processed(1)` を呼ぶ。`set_total` は「映像フレーム数 + 音声 PCM フレーム / 1 Opus フレームあたり PCM サンプル数」など両系統を反映する。

## 完了条件

音声のみ MP4 を入力しても、エンコード中に進捗バーが 0% → 100% まで動いて見える。映像あり MP4 でも従来通り進捗する。

## 解決方法

1. `src/encode/transcode.rs:159-167` の `encode_tracks` シグネチャの `progress: &JobProgress` を `Option<&JobProgress>` にして音声経路でも使えるようにする
2. `src/encode/transcode.rs:730-` (encode_audio_aac) と `src/encode/transcode.rs:779-` (encode_audio_opus) に `progress: Option<&JobProgress>` 引数を追加
3. 各 PCM フレーム処理後に `if let Some(p) = progress { p.add_processed(1); }` を追加
4. `src/encode/transcode.rs:154` の `set_total` を `max(video_samples.len(), audio_samples.len() / opus_frame_samples)` 等に変更

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0012 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0005 との並行**: 0005 (`encode_tracks` の部分成功対応) は `set_total` を変更する可能性があり、0012 と並行着手で conflict する。**0005 → 0012 の順** でコミットする、または**0005 と 0012 を統合** して 1 コミットにする。
- **0007 で整備される基盤**: `tests/test_encode.rs` への音声のみ MP4 smoke test 追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **0032/0046 との関係**: `tracing` 関連の追加なし。0032 (`add-tracing-logging`) / 0046 (`refactor-remove-tracing-init`) の判断に影響しない。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
