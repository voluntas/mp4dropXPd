# polling 100ms 固定間隔をイベント駆動化

- Priority: Low
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

`src/app.rs:196-209` の 100ms 固定 polling を `tokio::sync::watch` または channel ベースのイベント駆動に置き換える。

## 優先度根拠

短時間ファイル (1 秒以下) で進捗が 0% → 100% に一気に飛ぶ。100ms ごとの `cx.update` で GPUI スレッドを起こす負荷。

## 現状

`src/app.rs:196-209`:

```rust
loop {
    cx.background_executor()
        .timer(std::time::Duration::from_millis(100))
        .await;
    let total: f64 = progresses_for_poll.iter().map(|p| p.ratio()).sum::<f64>() / job_count as f64;
    let _ = this.update(cx, |view, cx| {
        view.state.progress = total;
        cx.notify();
    });
    if done_for_encode.load(Ordering::SeqCst) { break; }
}
```

## 設計方針

`JobProgress` に `tokio::sync::Notify` を持たせ、進捗更新時に `notify.notify_one()` を呼ぶ。ポーリングループは `notify.notified().await` で待機する。

## 完了条件

進捗更新時のみ UI が更新される。100ms ポーリングによる CPU 負荷がなくなる。短時間ファイルでも進捗が滑らかに動く。

## 解決方法

1. `src/encode/mod.rs:24-62` の `JobProgress` に `notify: Arc<tokio::sync::Notify>` フィールドを追加
2. `set_total` / `add_processed` 内で `self.notify.notify_one()` を呼ぶ
3. `src/app.rs:196-209` の polling ループを `loop { let _ = self.notify.notified().await; /* UI 更新 */ if done { break; } }` に置換

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0028 → 0029 (本 issue)** の順で develop に直接コミットする。0028 (進捗カウント位相) 完了後に着手。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0012 との関係**: 0012 (音声進捗) と並行可。`JobProgress` の API 拡張 (notify フィールド追加) は 0012 と衝突しない。
- **0032/0046 との関係**: 0032 (`add-tracing-logging`) と 0046 (`refactor-remove-tracing-init`) は tracing 設定を扱う。本 issue は `JobProgress` のみで、tracing とは独立。並行可。
- **0007 で整備される基盤**: `tests/test_app.rs` への通知ベースの進捗テスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
