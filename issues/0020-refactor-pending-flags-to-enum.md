# pending_* フラグ 5 種 + Option 2 種で「フラグ地獄」状態管理

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0019 → 0020 (本 issue)** の順で develop に直接コミットする。0019 (SharedState の Entity 化) 完了後に着手。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0011 との関係**: 0011 (`bug-pending-exit-abort`) は `pending_exit` フラグに abort 機構を追加する。0020 で `pending_exit` フラグが `AppCommand::Exit` に変わるため、**0011 → 0020 の順** でコミットするか、**0011 と 0020 を統合** して 1 コミットにする。0011 を 0020 完了後に re-polish する必要あり。
- **0007 で整備される基盤**: `tests/test_app.rs` への enum パターンのテスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
