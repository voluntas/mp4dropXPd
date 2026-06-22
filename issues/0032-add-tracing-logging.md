# ログ出力の欠如 (tracing 0 件)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/add-tracing-logging
- Polished:
- Reporter:

## 目的

`src/encode` 内に `tracing::info!` / `error!` / `warn!` を追加し、デバッグ容易性を上げる。

## 優先度根拠

shiguredo-rust 規約「ログは tracing を使うこと」を満たしていない。`tracing_subscriber` の init は main.rs に存在するが、発行する側が皆無。エンコード失敗時にログが残らないため、運用時のデバッグが困難。

## 現状

`src/` 全体で `tracing::*!` マクロの呼び出しが 0 件。`tracing_subscriber::fmt()...init()` のみが `src/main.rs:11-15` に存在 (デッドインフラ)。

## 設計方針

主要分岐にログを追加する。

- エンコード開始 / 終了
- demux 失敗
- エンコード失敗
- 進捗が長時間停滞 (例: 5 分間 0% 状態)
- 入力 MP4 のトラック数・サイズ

メッセージは英語 (AGENTS.md「ログメッセージは全て英語」)、`RUST_LOG=mp4dropxpd=debug` で有効化されるレベル。

## 完了条件

- エンコード開始 / 終了がログに残る
- 失敗時に `tracing::error!` で十分な情報 (エラー型、入力ファイル、ステップ) が残る
- `RUST_LOG=mp4dropxpd=debug cargo run` でデバッグログが流れる

## 解決方法

1. `src/encode/transcode.rs:96` の `let input_data = std::fs::read(input)?;` の直後に `tracing::info!(file = %input.display(), size = input_data.len(), "encode started");`
2. 各 `?` 演算子の直前に `tracing::error!(error = %e, "encode failed at step X");` を追加
3. `transcode` 関数の成功終了時に `tracing::info!("encode finished");`
4. `src/main.rs:12-14` の `tracing_subscriber::init()` にデフォルトレベル `LevelFilter::INFO` を設定
