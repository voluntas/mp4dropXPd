# 3 ウィンドウで recipe / settings が重複 + SharedGlobal 手書き pub/sub

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-shared-state-to-entity
- Polished:
- Reporter:

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
