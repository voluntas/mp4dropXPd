# エンコード中の quit で detach タスクが走り続ける

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0011 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0002 との連携**: 0002 (`drain_vt_encoder` タイムアウト) は `spawn_blocking` 内の `std::thread::sleep` を中断できないため、drain 中の `pending_exit` でも 30 秒待つ。0011 で `Task::abort()` しても `spawn_blocking` 内のスレッドは走り続ける。**0002 → 0011 の順** でコミットする (0002 のタイムアウト短縮が前提)。
- **0004 との並行**: 0004 (`TempFile` 導入) で「abort 時に部分成功ファイルが残る可能性」を `TempFile::drop` でカバーできる。0011 の abort 機構と 0004 の TempFile が組み合わさって安全な abort 動作になる。**0004 → 0011 の順** でコミットするか、**0004 と 0011 を統合** して 1 コミットにする。
- **0007 で整備される基盤**: `tests/test_app.rs` (UI テスト) への abort シナリオのテスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。abort シナリオのテストは UI 操作を含むため、UI テストフレームワーク (`gpui` のテストユーティリティ等) のセットアップも必要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### 不具合修正` サブセクション (0009 で確立) に `[FIX] エンコード中の quit でタスクを abort` を 1 行で追記する。
