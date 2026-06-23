# tokio JoinError を format!("Task: {e:?}") で UI に出す

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

panic 時の `JoinError` (バックトレース・メッセージを含む) を UI に平文で出さず、ログには詳細を残す。

## 優先度根拠

`shiguredo-no-secrets` 規約 (クレデンシャルを UI に出さない) に対する潜在的な漏洩リスク。panic バックトレースにクレデンシャルが含まれる可能性。

## 現状

`src/app.rs:226`:

```rust
Err(e) => format!("Task: {e:?}"),
```

`src/encode/mod.rs:94-97`:

```rust
Err(e) => out.push((
    EncodeJob::new(PathBuf::new(), PathBuf::new(), EncodeRecipe::default()),
    Err(Error::Message(format!("encode task failed: {e}"))),
)),
```

## 設計方針

UI 表示は `e.is_panic()` 判定で一律文言にする。デバッグ用に `tracing::error!` でログにバックトレースを残す。

## 完了条件

panic 時に UI には `"エンコードタスクがパニックしました"` 等のユーザーフレンドリーな文言が表示され、ログには完全なバックトレースが記録される。クレデンシャルが UI に露出しない。

## 解決方法

`src/app.rs:226` を以下のように修正する。

```rust
Err(e) => {
    if e.is_panic() {
        format!("エンコードタスクがパニックしました")
    } else {
        format!("エンコードタスクが失敗しました")
    }
}
```

`src/encode/mod.rs:94-97` のエラー時も同様に `tracing::error!` でログにバックトレースを残しつつ、UI には簡潔なメッセージを渡す。

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0008 → 0017 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0008 との関係**: 0008 (`bug-ui-strings-japanese`) は `src/app.rs:226` の `format!("Task: {e:?}")` を 0008 のスコープから外して 0017 に完全移管する取り決めがある。**0008 → 0017 の順** でコミットし、0017 で UI 文言 (`"エンコードタスクがパニックしました"`) を最終決定する。0008 では `src/app.rs:226` を翻訳しない。
- **shiguredo-no-secrets 規約**: 機密情報 (クレデンシャル等) を UI に出さない規約。本 issue で panic バックトレースが UI に出ないようにする。実装前に `shiguredo-no-secrets` スキル参照。
- **0007 で整備される基盤**: `tests/test_app.rs` (UI テスト) への panic / 通常エラー時の UI 表示テスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない (`tracing::error!` でログにバックトレースを残すのみ)。0018 への影響なし。
- **0032/0046 との関係**: `tracing::error!` の出力先確保は 0032 (`add-tracing-logging`) / 0046 (`refactor-remove-tracing-init`) の判断に従う。0032 が完了するまではログ出力は無視される可能性があるが、本 issue のスコープではログ呼び出しを入れるだけで完結する。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### 不具合修正` サブセクション (0009 で確立) に `[FIX] panic 時の JoinError を UI に直接出さず、簡潔な文言に置換 (詳細はログに記録)` を 1 行で追記する。
