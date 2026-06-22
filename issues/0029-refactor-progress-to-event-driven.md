# polling 100ms 固定間隔をイベント駆動化

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-progress-to-event-driven
- Polished:
- Reporter:

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
