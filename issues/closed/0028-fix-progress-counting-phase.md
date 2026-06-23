# 進捗カウンタがエンコード投入時点で先行

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

進捗カウンタをエンコード完了ベースでカウントするよう修正する。

## 優先度根拠

現状は `encoder.encode()` の Ok 返却直後に `progress.add_processed(1)` を呼ぶため、UI 進捗が実エンコード完了より先行する。drain / finish 後に完了したフレーム数でカウントすべき。

## 現状

`src/encode/transcode.rs:400, 482, 589` で `progress.add_processed(1)` を `encoder.encode()` 直後に呼んでいる。

## 設計方針

進捗を (a) decode phase / (b) encode phase / (c) drain phase の 3 段階に分け、フェーズ境界で明示的に `add_processed` を呼ぶ。VT エンコーダは `drain_vt_encoder` 完了時に `encoded_frames.len()` を加算。SVT-AV1 は `encoder.next_frame()` で取得できたフレーム数。

## 完了条件

UI の進捗バーが実エンコード完了と一致する。短時間ファイル (1 秒以下) で進捗が 0% → 100% に飛ぶ現象が緩和される。

## 解決方法

`src/encode/transcode.rs:400, 482, 589` の `progress.add_processed(1)` を `drain_vt_encoder` / `encoder.next_frame()` ループ後に移動。`src/encode/transcode.rs:404-409, 486-491` で `encoded_frames.len()` を `progress.set_total(encoded_frames.len())` のように再計算し、実際の完了フレーム数で `add_processed` を呼ぶ。

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0028 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0012 との関係**: 0012 (`bug-audio-only-progress`) は音声側の進捗を扱う。本 issue は映像側の進捗 (drain 完了ベース) で、0012 とは独立した修正領域。並行可。
- **0029 との関係**: 0029 (`refactor-progress-to-event-driven`) は進捗をイベント駆動化する。本 issue は進捗カウントの位相を変えるだけで、0029 とは独立。**0028 → 0029 の順** でコミットする。
- **0007 で整備される基盤**: `tests/test_encode.rs` への進捗テスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
