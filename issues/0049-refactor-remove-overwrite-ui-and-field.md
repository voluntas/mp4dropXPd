# AppSettings::overwrite 設定と関連 UI 3 箇所の削除 (統合判断版)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-overwrite-ui-and-field
- Polished:
- Reporter:

## 目的

#0010 (overwrite 機能実装) と #0041 (overwrite UI 削除) の判断を統合し、削除する。

## 優先度根拠

#0010 で機能実装する場合: overwrite フラグを `EncodeJob` に追加し、`transcode` 内で `OpenOptions::new().create_new(!overwrite)` を使う。UI は保持。

#0010 で機能実装しない場合: 機能しない UI は削除 (#0041)。

この issue は #0010 / #0041 の判断を補助するための統合 issue。

## 完了条件

#0010 で実装する → #0041 を close し、この issue も close。

#0010 で実装しない → #0010 を close し、#0041 とこの issue を実施。

## 解決方法

#0010 (overwrite 機能実装) の判断を待つ。
