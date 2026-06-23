# AppSettings::overwrite 設定と関連 UI 3 箇所の削除 (統合判断版)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0010 → 0049 (本 issue)** の順で develop に直接コミットする。0010 (`overwrite` 機能実装) の判断が出てから着手。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0010 との関係**: 0010 で `overwrite` 機能を実装する場合、本 issue (0049) は close。0010 で実装しない場合、0041 と 0049 を統合して `overwrite` フィールドと UI を削除する。**0010 の判断を待つ**。
- **0041 との関係**: 0041 (`refactor-remove-dead-overwrite-ui`) と 0049 は重複している (両者とも `overwrite` 削除を扱う)。**0041 と 0049 を統合** して 1 issue に集約する。0049 側で `overwrite` 削除 + 関連 refactor (0024 の `recipe/output_dir` dead code 削除との統合等) を行う。
- **0008 との関係**: 0008 (UI 文字列日本語化) は `src/app.rs:678, 680` の `"● Overwrite output"` 翻訳を保留している (0010/0041/0049 の判断待ち)。**0010 → 0008 → 0049** の順でコミットするか、0010 で `overwrite` を残す判断なら 0008 で翻訳、0049 で削除なら 0008 翻訳も削除。
- **0001-0008 との並行**: 本 issue は `src/settings.rs` と `src/app.rs` の修正で、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: 新規テスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] AppSettings::overwrite と関連 UI を削除 (0010 で機能実装しない場合)` を 1 行で追記する。
