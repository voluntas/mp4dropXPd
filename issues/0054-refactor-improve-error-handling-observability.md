# improve error handling and observability (tracing + remove empty placeholder)

- Priority: Medium
- Created: 2026-06-23
- Completed:
- Model: opencode/deepseek-v4-pro
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`src/encode/transcode.rs` に `tracing::info!` / `tracing::error!` を追加し、`src/encode/mod.rs` の `run_jobs_async` における panic 時の空 `EncodeJob` プレースホルダを削除することで、エラーハンドリングと可観測性を改善する。closed issue 0032 と 0047 を統合した issue。

## 優先度根拠

エンコード失敗時にログが一切残らず、panic 発生時に空の `EncodeJob` が作成される現在の実装は運用時のデバッグを著しく困難にする。実装品質に直結するため Medium とする。

## 現状

- `src/encode/transcode.rs`: `tracing` ログが一切存在せず、エラー情報は `Error::Message` 経由の文字列のみに依存している
- `src/encode/mod.rs:94-97`: `run_jobs_async` で `JoinError` 発生時に空の `EncodeJob::new(PathBuf::new(), PathBuf::new(), EncodeRecipe::default())` をプレースホルダとして `push` している。呼び出し元 (`app.rs`) は `Result` のみを使用しジョブ内容は無視するため、panic が事実上握りつぶされている
- `src/main.rs:12-14`: `tracing_subscriber` の `init` は存在するが、デフォルトログレベルの設定がなく `RUST_LOG` 環境変数に完全依存している

## 設計方針

- transcode の主要ステップ（開始、demux、エンコード、mux、完了）に `tracing::info!` を追加する
- エラー発生箇所に `tracing::error!` を追加し、エラー内容と呼び出し元ファイル情報を残す
- `src/main.rs` の `tracing_subscriber` 初期化にデフォルトレベル `LevelFilter::INFO` を設定する
- `run_jobs_async` の `JoinError` 分岐で空 `EncodeJob` を生成せず、`?` で早期 return して呼び出し元にエラーを伝播する

## 完了条件

- transcode の開始 / 終了が `tracing::info!` でログに残る
- 各エラーステップで `tracing::error!` により十分な情報 (エラー内容、ファイル、ステップ) が残る
- `RUST_LOG=mp4dropxpd=debug` でデバッグログが流れる
- `RUST_LOG` 未設定時も `LevelFilter::INFO` により開始 / 終了ログがデフォルトで出力される
- panic 時に空の `EncodeJob` が作成されず、`JoinError` が呼び出し元に伝播する

## 解決方法

1. `src/encode/transcode.rs`:
   - `transcode` 関数の先頭 (`std::fs::read(input)` 直後) に `tracing::info!(file = %input.display(), size = input_data.len(), "encode started");` を追加
   - demux エラー分岐 (L132) に `tracing::error!` を追加
   - `encode_tracks` 呼び出しのエラー分岐 (L159-166) に `tracing::error!` を追加
   - `write_mp4` 呼び出しのエラー分岐 (L170) に `tracing::error!` を追加
   - transcode 関数の成功終了時に `tracing::info!("encode finished");` を追加
2. `src/main.rs`:
   - `tracing_subscriber::EnvFilter::from_default_env()` に `.add_directive(tracing::Level::INFO.into())` 相当のデフォルトレベルを設定
3. `src/encode/mod.rs`:
   - `run_jobs_async` の `while let Some(joined) = set.join_next().await` ループ内の `match joined { ... Err(e) => ... }` を `Err(e) => return Err(Error::Message(format!("encode task panicked: {e}")))` に変更し、`Vec` の戻り値から `Result` に変更するか、あるいは関数シグネチャ自体を見直す
