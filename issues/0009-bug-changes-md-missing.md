# CHANGES.md 不在 (shiguredo-changelog 規約違反)

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

`shiguredo-changelog` 規約に従った `CHANGES.md` を作成し、変更履歴の記録基盤を整備する。

## 優先度根拠

AGENTS.md 40-42 行「変更履歴を記載するときは `shiguredo-changelog` スキルを参照すること」と shiguredo-changelog スキル 16-18 行「変更履歴は `CHANGES.md` に記載すること」「未リリースの変更は `## develop` セクションに追記すること」は CHANGES.md ファイルが前提。**0001-0008 の 8 件すべてが 0009 を前段依存として明示** しており、0009 が無ければ 8 件が着手できない。

## 現状

リポジトリルートに `CHANGES.md` が存在しない。`.markdownlint.jsonc` は存在するが `prek.toml` の 5 フック (trailing-whitespace / end-of-file-fixer / check-toml / cargo-fmt / cargo-clippy) には含まれていない。`git log` のコミットは 3 件 (`9c48214` issues/ ディレクトリ作成、`7c2af80` インポート、`fedd1d3` Initial commit)。

## 設計方針

`shiguredo-changelog` スキルの規約を `CHANGES.md` に適用する:

- ファイル先頭は `# 変更履歴` (shiguredo 慣例)
- 直下に `## develop` セクション (未リリース変更)
- 各エントリは `- [種別] 変更内容を〜するという形で書く` (shiguredo-changelog 31-32 行)
- 種別: `[CHANGE]` / `[ADD]` / `[UPDATE]` / `[FIX]` の 4 種 (shiguredo-changelog 22-29 行)
- エントリ順序: `CHANGE → ADD → UPDATE → FIX` (shiguredo-changelog 35 行)
- 機能に直接影響しない変更は `### misc` サブセクション (shiguredo-changelog 36 行) — 本 issue では使用しない
- 担当者は次の行に `- @ユーザー名` (shiguredo-changelog 37-38 行) — 本 issue では `@voluntas` を次の行に記載する
- 0001-0008 が追記する `[FIX]` エントリは `## develop` 内の `### 不具合修正` サブセクションに集約する (0001-0008 の `## 不具合修正` 表現を尊重しつつ、shiguredo-changelog 規約の 4 種別に沿う)
- コミット hash は記載しない (shiguredo-issues 39-43 行「CHANGES.md に issue 番号を書いてはいけない」と整合)

初回エントリ:

- `[ADD] CHANGES.md を作成し変更履歴の記録基盤を整備する`
  - `@voluntas`

## 完了条件

- `CHANGES.md` がリポジトリに存在し、`# 変更履歴` タイトル + `## develop` セクション + `- [ADD] ...` 初回エントリ + `- @voluntas` 担当者行が記載されている
- `.markdownlint.jsonc` の 4 ルール (MD004 / MD013 / MD024 / MD036) を満たす
- 0001-0008 が `[FIX] ...` 種別のエントリを `### 不具合修正` サブセクションに追記できる構造になっている
- `cargo clippy --all-targets -- -D warnings` (`prek.toml:28`) を通過する (CHANGES.md 編集のみなので影響なしだが確認)
- `prek` の 5 フックがローカルで 0 警告で完了する

## 解決方法

### 1. `CHANGES.md` 新規作成

リポジトリルートに以下の内容で `CHANGES.md` を作成:

```markdown
# 変更履歴

## develop

### 不具合修正

### misc

### 追加

- [ADD] CHANGES.md を作成し変更履歴の記録基盤を整備する
  - @voluntas
```

### 2. コミット

`shiguredo-git` 規約に従い develop に直接コミット:

- コミットメッセージ: `0009 CHANGES.md を作成する`
- 本文: なし
- 1 issue = 1 コミット原則

実装時の確認手順:

- **依存順序**: 本 issue は **0007 (テスト基盤整備) → 0009 (本 issue) → 0001-0008** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで 0001-0008 は着手不可。
- **0001-0008 との関係**: 0001-0008 の `## 解決方法 > 実装時の確認手順 > CHANGES.md 追記` サブ項目は、本 issue で確立した `## develop > ### 不具合修正` サブセクションに `[FIX] ...` 種別のエントリを追記する形となる。0001-0008 の「`## 不具合修正`」表現は本 issue で正式サブセクションとして確立する。
- **0001-0008 の re-polish 必要性**: 0001-0008 のうち「`## 不具合修正`」表現と本 issue で確立した `### 不具合修正` サブセクションの整合を確認するため、0009 完了後に 0001-0008 を一括 re-polish する (もしくは各 issue 着手時に個別確認)。本 issue では「`## 不具合修正` 等に本修正が追記されている」 (`0001-0008` の表現) を「`### 不具合修正` サブセクションに `[FIX] ...` 種別のエントリを追記」と読み替える。
- **prek への markdownlint 追加**: 別 issue `0009-add-prek-markdownlint` に切り出す (本 issue のスコープ外)。`CHANGES.md` の行長・リスト形式・見出し整合性は手動確認とし、prek での自動強制は別 issue で扱う。
- **0032 / 0046 との関係**: 本 issue では `tracing` 関連の記載なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
