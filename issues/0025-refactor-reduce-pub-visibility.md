# pub 境界が過剰 (DropState / MenuWindow / OptionsWindow のフィールド)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-reduce-pub-visibility
- Polished:
- Reporter:

## 目的

`DropState` / `MenuWindow` / `OptionsWindow` のフィールド `pub` を private 化または `pub(crate)` にし、getter に置き換える。

## 優先度根拠

`pub` フィールドは外部からの無制限な書き換えを許し、UI 内部の不変条件 (status は SharedString、encoding と他フィールドの整合) を守れない。

## 現状

`src/app.rs:38-49, 95-99, 515-520, 806-811` で `recipe` / `settings` / `status` / `encoding` などが `pub`。

## 設計方針

フィールドを private 化し、コンストラクタ `new(...)` で受け取る。状態変更は `set_recipe(&mut self, ...)` 等の専用メソッド経由にし、整合性チェックを 1 箇所に集約する。

## 完了条件

`pub` フィールドが大幅に減る (private または `pub(crate)` に)。getter (`fn recipe(&self) -> &EncodeRecipe`) が提供される。

## 解決方法

1. `src/app.rs:38-49` (`DropState`) の `pub` を削除
2. `src/app.rs:95-99` (`DropWindow`) の `pub state: DropState` を `state: DropState` に
3. `src/app.rs:515-520` (`MenuWindow`) の `pub` を削除
4. `src/app.rs:806-811` (`OptionsWindow`) の `pub` を削除
5. `fn recipe(&self) -> &EncodeRecipe` / `fn settings(&self) -> &AppSettings` 等の getter を追加
6. `set_recipe(&mut self, recipe: EncodeRecipe)` 等の setter を追加
