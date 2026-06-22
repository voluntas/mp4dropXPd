# DropState::status + idle_hint のデッドコード削除

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-dropstate-status
- Polished:
- Reporter:

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
