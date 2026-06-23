# AppSettings::recipe / output_dir が完全な dead code (YAGNI 違反)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0024 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0019 との関係**: 0019 (`SharedState` の Entity 化) は `AppSettings` の利用箇所を変更する可能性。本 issue で `AppSettings` から 2 フィールド削除するため、0019 との並行は安全 (0019 は `AppSettings` 自体は削除しない)。
- **0041 との関係**: 0041 (`refactor-remove-dead-overwrite-ui`) は `AppSettings::overwrite` フィールド削除を扱う。0041 との並行は安全 (`overwrite` フィールドは本 issue の対象外)。
- **0007 で整備される基盤**: ドキュメント/フィールド削除のみなので新規テスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] AppSettings::recipe / output_dir (dead code) を削除` を 1 行で追記する。
