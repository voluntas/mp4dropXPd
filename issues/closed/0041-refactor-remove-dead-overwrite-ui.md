# AppSettings::overwrite フィールドと関連 UI 3 箇所の削除 (機能実装しない場合)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0010 → 0041 (本 issue)** の順で develop に直接コミットする。0010 (`overwrite` 機能実装) の判断が出てから着手。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0010 との関係**: 0010 で `overwrite` 機能を実装する場合、本 issue は close (0010 統合)。0010 で実装しない場合、本 issue で `overwrite` フィールドと UI を削除する。**0010 の判断を待つ**。
- **0008 との関係**: 0008 (UI 文字列日本語化) は `src/app.rs:678, 680` の `"● Overwrite output"` 翻訳を保留している (0010/0041/0049 の判断待ち)。**0010 → 0008 → 0041** の順でコミットするか、0010 で `overwrite` を残す判断なら 0008 で翻訳、0041 で削除なら 0008 翻訳も削除。
- **0049 との関係**: 0049 (`refactor-remove-overwrite-ui-and-field`) は本 issue と類似。本 issue (UI + field 削除) と 0049 (UI + field 削除 + 関連 refactor) は重複。**0041 と 0049 を統合** するか、0041 → 0049 の順で順次着手。
- **0001-0008 との並行**: 本 issue は `src/settings.rs` と `src/app.rs` の修正で、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: 新規テスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] AppSettings::overwrite フィールドと関連 UI を削除 (0010 で機能実装しない場合)` を 1 行で追記する。
