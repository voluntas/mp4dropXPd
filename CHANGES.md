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
- [FIX] エンコードタスクの panic 時に UI にユーザーフレンドリーな文言を表示する
  - @voluntas
- [UPDATE] 構造化エラー型を追加し主要な Error::Message 箇所を置換する (InvalidSampleRange / DrainTimeout / NoTrack / TrackEncode / Demux)
  - @voluntas
- [UPDATE] pending_* フラグ 4 種を AppCommand enum + VecDeque に置換する
  - @voluntas
- [UPDATE] encode_video_vt 共通関数を抽出し H.264/H.265 の重複を削減する
  - @voluntas
- [UPDATE] app.rs (1208 行) と transcode.rs (1340 行) を機能別モジュールに分割する
  - @voluntas
- [UPDATE] 進捗ポーリングを完全イベント駆動に変更し 100ms 固定待機を除去する
  - @voluntas
- [FIX] 入力音声サンプルの composition_time_offset が mux 時に破棄される問題を修正する
  - @voluntas
- [FIX] f64 ソートを整数ソートに置き換え長時間動画の順序破壊を防止する
  - @voluntas
- [FIX] 進捗カウントをエンコード実完了ベースに修正する
  - @voluntas
- [UPDATE] MuxWriter 構造体を導入し data_offset の手動管理をカプセル化する
  - @voluntas
- [UPDATE] エラーハンドリングと可観測性を改善する (tracing ログ追加 + デフォルトレベル設定 + panic 時処理改善)
  - @voluntas
- [UPDATE] async runtime と progress control を改善する (worker_threads 動的決定 + Notify ベース進捗待機)
  - @voluntas

### misc

- [UPDATE] AppSettings のデッドフィールドと DropState::status/idle_hint を削除する
  - @voluntas
- [UPDATE] EncodeRecipe のメソッドリファクタリング (リネーム + codec_tag + DEFAULT_VIDEO_TIMESCALE)
  - @voluntas
- [UPDATE] sample_entry に仕様参照を追加し README に UI 役割分担を明記する
  - @voluntas
- [UPDATE] window_remove ヘルパーを削除し MenuWindow に多重起動防止を追加する
  - @voluntas
- [UPDATE] DropState / MenuWindow / OptionsWindow の pub フィールドを private 化する
  - @voluntas

### 追加

- [ADD] CHANGES.md を作成し変更履歴の記録基盤を整備する
  - @voluntas
- [ADD] テスト基盤を導入する (`proptest` 依存、`pbt/tests/`、`tests/` ディレクトリ、`.github/workflows/test.yml`、最初の PBT `prop_codec.rs`)
  - @voluntas
