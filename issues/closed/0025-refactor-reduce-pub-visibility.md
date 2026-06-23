# pub 境界が過剰 (DropState / MenuWindow / OptionsWindow のフィールド)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0025 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0019, 0020 との関係**: 0019 (`SharedState` の Entity 化) と 0020 (`pending_*` フラグ enum 化) は `SharedState` の構造を変える。本 issue で `pub` フィールドを private 化する対象は 0019/0020 の構造変更と関連する。**0019/0020 → 0025 の順** でコミットする。
- **0024 との関係**: 0024 (`AppSettings::recipe/output_dir` 削除) と並行可 (別フィールド)。
- **0007 で整備される基盤**: `tests/test_app.rs` への getter/setter のテスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
