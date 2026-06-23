# ログ出力の欠如 (tracing 0 件)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0032 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0046 との関係**: 0046 (`refactor-remove-tracing-init`) は `tracing_subscriber` の init を削除する提案。0032 と 0046 は逆方向の作業。**0032 → 0046 の判断** は別 issue (`triage-issues`) で扱う。0032 着手時は init を残す前提で進める。
- **0001-0008 との関係**: 0001, 0002, 0003 は本 issue で追加する `tracing::warn!` を呼び出している。0001-0008 完了後に本 issue でログ呼び出しを追加するか、並行で進めるかは判断。**0001-0008 と 0032 を並行** で進めるのが conflict しない (0001-0008 のログ呼び出しは 0032 とは独立)。
- **0007 で整備される基盤**: テストは `tracing-test` 等のテストユーティリティが必要なら 0007 の `Cargo.toml` 編集と同時に。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### 追加` サブセクション (0009 で確立) に `[ADD] tracing::info!/error!/warn! を src/encode に追加` を 1 行で追記する。
