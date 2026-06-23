# 変更履歴

## develop

### 不具合修正

- [FIX] 映像/音声トラックを含まない MP4 入力時の破損ファイル生成を防止する
  - @voluntas
- [FIX] 破損 MP4 入力時の data_offset + data_size 範囲外 panic を Result エラーに変換する
  - @voluntas
- [FIX] drain_vt_encoder の永久 busy-wait を 30 秒タイムアウトに変換する
  - @voluntas
- [FIX] AV1 (dav1d) デコードのエラー握り潰しと最終フレームドレイン漏れを修正する
  - @voluntas
- [FIX] write_mp4 を tmp + atomic rename 方式に変更し出力ファイル破損を防止する
  - @voluntas
- [FIX] encode_tracks を部分成功対応に変更する (映像/音声を独立評価、片方失敗時に残りを保持)
  - @voluntas
- [FIX] UI 文字列を全て日本語化する (技術用語例外を除く)
  - @voluntas
- [FIX] AppSettings::overwrite 設定をエンコード経路に配線する
  - @voluntas
- [FIX] エンコード中の quit で detach タスクが走り続ける問題を修正する (Task を保持し abort 可能に)
  - @voluntas
- [FIX] 音声のみ MP4 で進捗バーが 0% 固定になる問題を修正する
  - @voluntas
- [FIX] 英語コメントを全て日本語に書き換える (AGENTS.md 準拠)
  - @voluntas
- [FIX] 入力音声サンプルの composition_time_offset が mux 時に破棄される問題を修正する
  - @voluntas

### misc

- [UPDATE] AppSettings のデッドフィールドと DropState::status/idle_hint を削除する
  - @voluntas
- [UPDATE] EncodeRecipe のメソッドリファクタリング (リネーム + codec_tag + DEFAULT_VIDEO_TIMESCALE)
  - @voluntas

### 追加

- [ADD] CHANGES.md を作成し変更履歴の記録基盤を整備する
  - @voluntas
- [ADD] テスト基盤を導入する (`proptest` 依存、`pbt/tests/`、`tests/` ディレクトリ、`.github/workflows/test.yml`、最初の PBT `prop_codec.rs`)
  - @voluntas
