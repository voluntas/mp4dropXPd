# AppSettings::overwrite 設定がエンコード経路に未配線

- Priority: High
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

UI の「上書き保存」設定 (`AppSettings::overwrite`) をエンコード経路に配線し、既存ファイルの扱いを制御する。`overwrite = false` のときは出力先ファイルが既存なら `Err` を返し、`overwrite = true` のときは今まで通り truncate する。

## 優先度根拠

ユーザーは UI で ON/OFF できながら、実際のファイル上書き動作は変わっていない (UX バグ)。設定が機能していない。

## 現状

- `src/settings.rs:14` で `pub overwrite: bool` 定義
- `src/app.rs:663-682, 949-967` で UI トグル
- `src/encode/transcode.rs:995` で `File::create(output)?` が無条件 truncate (上書き)

`EncodeJob` および `transcode` 関数に `overwrite` フラグが渡されていない。

## 設計方針

`EncodeJob` に `overwrite: bool` フィールドを追加し、`transcode` 内で `OpenOptions::new().create(!overwrite).create_new(!overwrite).truncate(overwrite)` を使う。`overwrite = true` のときは既存ファイルを truncate、`overwrite = false` のときは `create_new(true)` で既存ファイルがあれば `AlreadyExists` エラーを返す。

エラー表現は `Error::Io(std::io::ErrorKind::AlreadyExists)` (`error.rs:42-46` の `From<std::io::Error>` 経由) で十分。`Error::Message` を新規追加しないため、0018 (構造化エラー化) への影響なし。

## 完了条件

- `overwrite = false` のとき、出力先ファイルが既に存在すれば `Err(Error::Io(std::io::ErrorKind::AlreadyExists))` 相当が返る
- `overwrite = true` のとき、今まで通り truncate する
- 0004 (atomic rename) と統合して 1 コミットにまとめるか、別 commit として develop に直接コミットする
- `tests/test_encode.rs` に両ケース (`overwrite = true` / `overwrite = false` × 既存ファイルあり/なし) のテストが追加されている
- `cargo clippy --all-targets -- -D warnings` (`prek.toml:28`) を通過する
- [FIX] AppSettings::overwrite をエンコード経路に配線

## 解決方法

1. `src/encode/job.rs` の `EncodeJob` に `overwrite: bool` フィールドを追加
2. `src/app.rs:170` の `EncodeJob::new` 呼び出しに `self.state.settings.overwrite` を渡す
3. `src/encode/mod.rs:64-77` の `encode_file_async` シグネチャに `overwrite: bool` を追加
4. `src/encode/transcode.rs:995` を `let mut file = std::fs::OpenOptions::new().write(true).create(!overwrite).create_new(!overwrite).truncate(overwrite).open(output)?;` に変更
5. 必要に応じて `default_output_path` (`output.rs:5`) で衝突時の suffix 付与ロジックも追加

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0004 (atomic rename) → 0010 (本 issue)** の順で develop に直接コミットする。
- **0004 との統合判断**: 0004 の `TempFile` 導入後、本 issue で `OpenOptions` による overwrite 制御を追加する。0004 → 0010 の順または統合。
- **0007 で整備される基盤**: `tests/test_encode.rs` へのテスト追加は 0007 完了後に行う。
- **0018 との関係**: `Error::Io` 経由のため 0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
