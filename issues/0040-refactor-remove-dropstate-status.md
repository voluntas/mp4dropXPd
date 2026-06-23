# DropState::status + idle_hint のデッドコード削除

- Priority: Low
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

`DropState::status` フィールドと `DropState::idle_hint` メソッドおよび関連呼び出しを削除する。

## 優先度根拠

`status` フィールドは 5 箇所 (line 56, 69-76, 88, 160, 233) で書き込まれるが、`Render for DropWindow` (app.rs:329-510) で一切表示されていない。`idle_hint` メソッド (line 64-78) の唯一の目的は `self.status` 更新。完全にデッド。

## 現状

`src/app.rs:42` (`pub status: SharedString`) + `src/app.rs:51-78` (`DropState::new` / `idle_hint`) + 呼び出し元 (`src/app.rs:90, 115, 137`)。

注: `OptionsWindow.status` フィールド (`src/app.rs:809, 819, 1181, 1191`) は別物。Save 時の "Saved" フィードバックに使われており削除不可。

## 設計方針

`DropState` から `status` フィールドを削除。`idle_hint` メソッドも削除。`add_paths` (line 80-92) 内の `self.status = ...` 代入と `self.idle_hint()` 呼び出しも削除。`poll_shared` (line 115) の `self.state.idle_hint()` 呼び出しも削除。

## 完了条件

`DropState::status` フィールド、`idle_hint` メソッド、関連呼び出しがすべて削除される。`cargo build` / `cargo clippy` が通る。

## 解決方法

1. `src/app.rs:42` の `pub status: SharedString,` を削除
2. `src/app.rs:51-78` の `DropState::new` から `status: "Drop MP4 here · 右クリックで設定".into(),` を削除
3. `src/app.rs:64-78` の `fn idle_hint(&mut self) { ... }` を削除
4. `src/app.rs:88` の `self.status = "MP4 only".into();` を削除
5. `src/app.rs:90` の `self.idle_hint();` を削除
6. `src/app.rs:115, 137` の `self.state.idle_hint();` を削除
7. `src/app.rs:160` の `self.state.status = ...` を削除
8. `src/app.rs:233` の `view.state.status = summary.into();` を削除

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0040 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0039 との関係**: 0039 (`let _status = ...` 削除) は本 issue の一部に含まれる。**0039 と 0040 を統合** して 1 コミットにする。
- **0008 との関係**: 0008 (UI 文字列日本語化) は `src/app.rs:88` の `"MP4 only"` 翻訳を含むが、0040 でこの行が削除されるため 0008 の翻訳対象から外れる。**0040 → 0008 の順** でコミットするか、**0008 と 0040 を統合** して 1 コミットにする。
- **0001-0008 との並行**: 本 issue は `src/app.rs` の 8 箇所変更で、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: 新規テスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
