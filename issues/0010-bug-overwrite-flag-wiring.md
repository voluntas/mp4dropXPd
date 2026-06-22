# AppSettings::overwrite 設定がエンコード経路に未配線

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-overwrite-flag-wiring
- Polished:
- Reporter:

## 目的

UI の「上書き保存」設定 (`AppSettings::overwrite`) をエンコード経路に配線し、既存ファイルの扱いを制御する。

## 優先度根拠

ユーザーは UI で ON/OFF できながら、実際のファイル上書き動作は変わっていない (UX バグ)。設定が機能していない。

## 現状

- `src/settings.rs:14` で `pub overwrite: bool` 定義
- `src/app.rs:663-682, 949-967` で UI トグル
- `src/encode/transcode.rs:995` で `File::create(output)?` が無条件 truncate (上書き)

`EncodeJob` および `transcode` 関数に `overwrite` フラグが渡されていない。

## 設計方針

`EncodeJob` に `overwrite: bool` フィールドを追加し、`transcode` 内で `OpenOptions::new().create_new(!overwrite)` を使う。

## 完了条件

`overwrite = false` のとき、出力先ファイルが既に存在すれば `Err(Error::Io(std::io::ErrorKind::AlreadyExists))` 相当が返る。`overwrite = true` のとき、今まで通り truncate する。

## 解決方法

1. `src/encode/job.rs` の `EncodeJob` に `overwrite: bool` フィールドを追加
2. `src/app.rs:170` の `EncodeJob::new` 呼び出しに `self.state.settings.overwrite` を渡す
3. `src/encode/mod.rs:64-77` の `encode_file_async` シグネチャに `overwrite: bool` を追加
4. `src/encode/transcode.rs:995` を `let mut file = std::fs::OpenOptions::new().write(true).create(!overwrite).create_new(!overwrite).truncate(overwrite).open(output)?;` に変更
5. 必要に応じて `default_output_path` (output.rs:5) で衝突時の suffix 付与ロジックも追加
