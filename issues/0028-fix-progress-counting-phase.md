# 進捗カウンタがエンコード投入時点で先行

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-progress-counting-phase
- Polished:
- Reporter:

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
