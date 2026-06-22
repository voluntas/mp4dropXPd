# issues/ ディレクトリと issues 整備 (shiguredo-issues 規約)

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/add-issues-directory
- Polished:
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
