# 変更履歴

## develop

### 不具合修正

- [FIX] 映像/音声トラックを含まない MP4 入力時の破損ファイル生成を防止する
  - @voluntas

### misc

### 追加

- [ADD] CHANGES.md を作成し変更履歴の記録基盤を整備する
  - @voluntas
- [ADD] テスト基盤を導入する (`proptest` 依存、`pbt/tests/`、`tests/` ディレクトリ、`.github/workflows/test.yml`、最初の PBT `prop_codec.rs`)
  - @voluntas
