# AppSettings::recipe / output_dir が完全な dead code (YAGNI 違反)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-dead-app-settings-fields
- Polished:
- Reporter:

## 目的

`AppSettings::recipe` / `output_dir` フィールドと関連初期化コードを削除する。

## 優先度根拠

AGENTS.md「Premature Optimization is the Root of All Evil」は YAGNI にも適用。`#[expect(dead_code)]` で「将来の設定永続化で使う」とコメントされているが、現状 read する箇所は皆無。YAGNI 違反。

## 現状

`src/settings.rs:9-13`:

```rust
#[expect(dead_code)]
pub recipe: EncodeRecipe,
#[expect(dead_code)]
pub output_dir: PathBuf,
```

`src/settings.rs:23-24` の `Default::default()` で `output_dir: PathBuf::from(".")` を初期化しているが、コードベース全体で `settings.recipe` / `settings.output_dir` の参照は 0 件。

## 設計方針

YAGNI に従い、フィールドを削除する。永続化が本当に必要になった時点で別 issue で実装する。

## 完了条件

`recipe` / `output_dir` フィールド、`Default` 内の対応する初期化、`#[expect(dead_code)]` 注釈がすべて削除される。`cargo build` / `cargo clippy` が通る。

## 解決方法

`src/settings.rs:9-13, 23-24` を以下のように修正する。

- `#[expect(dead_code)] pub recipe: EncodeRecipe,` の行を削除
- `#[expect(dead_code)] pub output_dir: PathBuf,` の行を削除
- `recipe: EncodeRecipe::default(),` の行を削除
- `output_dir: PathBuf::from("."),` の行を削除
