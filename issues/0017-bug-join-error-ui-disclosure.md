# tokio JoinError を format!("Task: {e:?}") で UI に出す

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-join-error-ui-disclosure
- Polished:
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
