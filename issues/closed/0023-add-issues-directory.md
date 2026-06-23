# issues/ ディレクトリと issues 整備 (shiguredo-issues 規約)

- Priority: Medium
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`shiguredo-issues` 規約に従って `issues/` ディレクトリと `SEQUENCE` ファイルを整備し、issue 管理体系を整える。

## 優先度根拠

AGENTS.md「issues を作成・管理するときは `shiguredo-issues` スキルを参照すること」は issues/ ディレクトリ管理を前提とする。今回 24 件の issue を作成する基盤となる。

## 現状

`issues/` ディレクトリは本日作成済みだが、`SEQUENCE` ファイルのみで他の issue 管理機能 (テンプレート、命名規則) は未整備。

## 設計方針

`shiguredo-issues` スキルに従って `issues/` 配下に以下を整備する。

- `SEQUENCE` ファイル (次の番号)
- `closed/` サブディレクトリ (解決済み)
- 命名規則: `{seqnum}-{category}-{short-description}.md`
- テンプレート: `shiguredo-issues` スキルに同梱されたものを利用

## 完了条件

`issues/` 配下が `shiguredo-issues` 規約に準拠した状態になっている。本 issue を含む 24 件の issue が `issues/` 直下に作成される。

## 解決方法

1. `shiguredo-issues` スキルを確認
2. `issues/SEQUENCE` ファイルの値を最新に保つ
3. 必要に応じて `issues/closed/` サブディレクトリを作成
4. 命名規則に従い、issue ファイルを配置

実装時の確認手順:

- **依存順序**: 本 issue は **0001-0008 より先** に着手する。具体的には `issues/` ディレクトリと `SEQUENCE` ファイルが既存 (本 issue 起票時点で既に 49 件の issue が作成済み) のため、本 issue は事実上「整備状況の確認」と「追加ディレクトリ (`closed/`) の作成」に絞られる。
- **0007 との関係**: 0007 (`bug-test-infrastructure-missing`) は `tests/` ディレクトリを整備する。0023 とは独立だが、リポジトリ管理 (issues + tests) として関連。
- **0009 との関係**: 0009 (`bug-changes-md-missing`) は `CHANGES.md` を整備する。0023 とは独立。
- **完了状態**: 本 issue 起票時点 (2026-06-22) で `issues/` ディレクトリと `SEQUENCE` (値: 50) は既に存在。49 件の issue も `issues/` 直下に作成済み (`git log: 9c48214 issues/ ディレクトリを作成し 49 件の issue を起票する`)。本 issue の作業は以下の点に限られる:
  - `issues/closed/` サブディレクトリの作成 (closed issue の移動先、まだ 0 件)
  - `issues/SEQUENCE` ファイルの最新化 (現状 50 で 0049 まで連番使用済み、最新は 50)
  - `.gitignore` の追加 (`issues/closed/.gitkeep` 等の空ディレクトリ維持)
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: ドキュメント/ディレクトリ整備のみなので影響なし。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[ADD] issues/closed/ サブディレクトリと issues 管理の整備` を 1 行で追記する。
