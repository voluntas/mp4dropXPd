# 3 ウィンドウで recipe / settings が重複 + SharedGlobal 手書き pub/sub

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

`DropState` / `MenuWindow` / `OptionsWindow` の `recipe` / `settings` 重複と `SharedGlobal` 手書き pub/sub を解消する。

## 優先度根拠

3 つの struct が同じ `recipe: EncodeRecipe` と `settings: AppSettings` を `pub` で保持し、`publish_recipe` / `publish_settings` で `SharedGlobal` に書き戻す手書きパターンは GPUI として非標準。GPUI の `Entity<T>` パターンに乗せ替える方が保守性が高い。

## 現状

`src/app.rs:38-49, 515-520, 806-811, 533-541, 825-834` で 3 つの struct が `recipe` / `settings` を重複保持。`SharedGlobal` (Mutex<SharedState>) を介して手書き pub/sub。

## 設計方針

`AppState { recipe, settings, ... }` を `cx.new_model` でエンティティ化し、各ウィンドウは `Entity<AppState>` を参照。GPUI の `cx.notify()` による再レンダリングで十分。

## 完了条件

`recipe` / `settings` の重複が解消される。`publish_recipe` / `publish_settings` ヘルパーが削除される。`SharedGlobal` が GPUI の `Entity<T>` パターンに置換される。

## 解決方法

1. `src/app/state.rs` (新規) に `AppState { recipe: EncodeRecipe, settings: AppSettings, ... }` を定義
2. `src/main.rs` (または `app.rs` の初期化) で `cx.new_model(|_| AppState::default())` してエンティティ化
3. `DropWindow`, `MenuWindow`, `OptionsWindow` から `recipe` / `settings` フィールドを削除し、`Entity<AppState>` を保持
4. `publish_*` ヘルパーを削除し、`cx.update_entity(&entity, |state, cx| { state.recipe = new_recipe; cx.notify(); })` パターンに置換

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0019 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0008 との関係**: 0008 (`bug-ui-strings-japanese`) は `src/app.rs` の UI 文字列を翻訳する。本 issue は `src/app.rs` を `src/app/{mod, state, drop_window, ...}.rs` に分割する。**0008 → 0019 の順** でコミットするか、**0008 と 0019 を統合** して 1 コミットにする。
- **0020 との関係**: 0020 (`refactor-pending-flags-to-enum`) は `SharedState` のフラグを enum に置換する。0020 は本 issue 完了後 (もしくは 0020 と本 issue を統合) に着手。SharedState の構造が本 issue で変わるため、0020 の前提となる。
- **0007 で整備される基盤**: `tests/test_app.rs` (UI テスト) への Entity パターンのテスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
