# pending_* フラグ 5 種 + Option 2 種で「フラグ地獄」状態管理

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-pending-flags-to-enum
- Polished:
- Reporter:

## 目的

5 つの `pending_*` bool フラグ + 2 つの `Option<T>` を `enum AppCommand` ベースの状態管理に置換する。

## 優先度根拠

`SharedState` のフラグ数が増えるたびに分岐を追加するリスク。`poll_shared` の直線的な if-else が読みにくい。`Drop` での値更新タイミングが型で表現できない。

## 現状

`src/app.rs:20-29`:

```rust
struct SharedState {
    pending_open_options: bool,
    pending_encode: bool,
    pending_clear: bool,
    pending_exit: bool,
    pending_close_menu: bool,
    latest_recipe: Option<EncodeRecipe>,
    latest_settings: Option<AppSettings>,
}
```

`src/app.rs:111-150` `poll_shared` で 5 つのフラグを直線的に舐めて消費。

## 設計方針

`enum AppCommand { OpenOptions, TriggerEncode, Clear, Exit, CloseMenu, UpdateRecipe(EncodeRecipe), UpdateSettings(AppSettings) }` を導入し、`SharedState.commands: VecDeque<AppCommand>` で送受信。`latest_recipe` / `latest_settings` は別管理にして値の取り違えを防ぐ。

## 完了条件

`pending_*` bool フラグがなくなる。`poll_shared` の if-else 連鎖が `match commands.pop_front()` パターンに置換される。

## 解決方法

1. `src/app.rs:20-29` を `commands: VecDeque<AppCommand>` を含む `SharedState` に置換
2. `src/app.rs:111-150` `poll_shared` を `while let Some(cmd) = s.commands.pop_front() { match cmd { AppCommand::OpenOptions => ..., AppCommand::TriggerEncode => ..., ... } }` に書き換え
3. `view.request(cx, |s| s.pending_xxx = true)` を `s.commands.push_back(AppCommand::Xxx)` に置換
