# AppSettings::overwrite フィールドと関連 UI 3 箇所の削除 (機能実装しない場合)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-dead-overwrite-ui
- Polished:
- Reporter:

## 目的

`AppSettings::overwrite` フィールドと関連 UI 3 箇所を削除する (issue #0010 で機能実装しない場合)。

## 優先度根拠

UI から設定可能だが、エンコード側 (`src/encode/transcode.rs:995` `File::create(output)`) では未参照。ユーザーが ON/OFF しても出力上書き動作は変わらない (UX バグ)。機能実装 (#0010) しない場合は削除。

## 現状

- `src/settings.rs:14, 25` (`pub overwrite: bool` フィールド + `Default` 初期化)
- `src/app.rs:663-682` (MenuWindow の "Overwrite output" トグル)
- `src/app.rs:949-967` (OptionsWindow の "Overwrite output" 設定行)

## 設計方針

#0010 (機能実装) と統合して判断する。実装するならこの issue はクローズ。実装しないなら削除。

## 完了条件

#0010 で実装する → この issue は close。#0010 で実装しない → この issue で削除。

## 解決方法

#0010 で実装する場合: この issue を close する。

#0010 で実装しない場合:

1. `src/settings.rs:14` の `pub overwrite: bool,` を削除
2. `src/settings.rs:25` の `overwrite: false,` を削除
3. `src/app.rs:662-682` の MenuWindow の "Overwrite output" トグル UI を削除
4. `src/app.rs:949-967` の OptionsWindow の "Overwrite output" 設定行を削除
5. `src/app.rs:1202` 周辺で `MenuWindow` の `w.settings = settings` に渡す前に `overwrite` フィールドを除外 (compile error 回避)
