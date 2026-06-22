# CHANGES.md 不在 (shiguredo-changelog 規約違反)

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/add-changes-md
- Polished:
- Reporter:

## 目的

`shiguredo-changelog` 規約に従った `CHANGES.md` を作成し、`## develop` セクションに変更履歴を記録する基盤を整備する。

## 優先度根拠

AGENTS.md「変更履歴を記載するときは `shiguredo-changelog` スキルを参照すること」は CHANGES.md への記載を前提とする。shiguredo プロジェクトの慣例として CHANGES.md は必須。

## 現状

リポジトリルートに `CHANGES.md` が存在しない。

## 設計方針

`shiguredo-changelog` スキルに従って `CHANGES.md` を作成し、現状の変更 (Initial commit からの 2 コミット: `fedd1d3 Initial commit` と `7c2af80 インポート`) を `## develop` セクションに記録する。

## 完了条件

`CHANGES.md` がリポジトリに存在し、`## develop` セクションに初版エントリが記載されている。

## 解決方法

`shiguredo-changelog` スキルを参照して `CHANGES.md` を作成。
