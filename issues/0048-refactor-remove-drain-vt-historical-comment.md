# drain_vt_encoder の過剰な過去履歴コメント削除

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-drain-vt-historical-comment
- Polished:
- Reporter:

## 目的

`src/encode/transcode.rs:520-523` の「以前は `empty_retries > 1000` で打ち切っていた」コメントを削除し、現状コードにあった簡潔な説明に置き換える。

## 優先度根拠

過去履歴 (現在は存在しない `empty_retries` 変数と `100ms` 閾値) の説明が残っており、現状コードと整合しない。コメントが現状の挙動を正確に表していない。

## 現状

`src/encode/transcode.rs:520-523`:

```rust
// VideoToolbox のエンコードは非同期で、finish() 後にフレームが順次届く。
// 以前は empty_retries > 1000 (合計 100ms) で打ち切っていたが、
// エンコーダ内部の遅延が大きいとフレームを欠落させる原因になるため、
// 期待フレーム数に達するまで無制限に待つ。
```

## 設計方針

現状コードの挙動を正確に反映した簡潔なコメントに置き換える (タイムアウトについては #0002 で対応)。

## 完了条件

過去履歴のコメントが削除され、現状コードと整合する簡潔なコメントに置き換わる。

## 解決方法

`src/encode/transcode.rs:520-523` を以下のように置き換える。

```rust
// VideoToolbox のエンコードは非同期で、finish() 後にフレームが順次届くため、
// expected フレームに達するまでポーリングする。
```

タイムアウト挙動は #0002 で対応。
