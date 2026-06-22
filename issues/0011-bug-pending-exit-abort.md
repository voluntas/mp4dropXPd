# エンコード中の quit で detach タスクが走り続ける

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-pending-exit-abort
- Polished:
- Reporter:

## 目的

`pending_exit` (`cx.quit()`) 時に進行中のエンコードタスクを中断できるようにする。

## 優先度根拠

`cx.spawn(...).detach()` で切り離されたタスクは `cx.quit()` 後も tokio ランタイム上で継続実行。`File::create` の truncate と `mux.append_sample` の途中書き出しが残る可能性。

## 現状

`src/app.rs:139-144, 180-240`:

```rust
if s.pending_exit {
    s.pending_exit = false;
    drop(s);
    cx.quit();
    return;
}
...
cx.spawn(async move |this, cx| {
    ...
})
.detach();
```

detach されたタスクは `this.update(cx, ...)` を app.rs:229-238 で呼ぶが、cx.quit() 後は view が破棄されている可能性が高く、`let _ =` でエラーを握り潰している。

## 設計方針

`Task<()>` を `detach()` せず `DropWindow` フィールドで保持し、`Drop` で `abort()` する。`pending_exit` 時に「エンコード中なら abort フラグを立てて quit を保留」する。

## 完了条件

エンコード中にユーザーが Exit を選んだとき、進行中のエンコードタスクが abort され、quit が実行される。出力ファイルが破損状態で残らない。

## 解決方法

`src/app.rs:95-99` の `DropWindow` 構造体に `encode_task: Option<Task<()>>` フィールドを追加。`schedule_encode` で `cx.spawn(...).detach()` の代わりに `let task = cx.spawn(...);` で受け取ってフィールドに保持。`pending_exit` 処理で `self.encode_task.take().map(|t| t.detach());` してから `cx.quit()`。`Drop` 実装で残ったタスクを abort。
