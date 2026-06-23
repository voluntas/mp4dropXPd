# async runtime と progress control の改善

- Priority: Low
- Created: 2026-06-23
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

`src/runtime.rs` のハードコード `worker_threads(4)` を動的決定に変更し、
`src/app.rs` の 100ms 固定 polling をイベント駆動 (`tokio::sync::Notify`) に置き換える。
0029 (進捗イベント駆動化) と 0036 (runtime worker_threads 動的決定) を統合する。

## 優先度根拠

Low。進捗表示の滑らかさと CPU コア数の有効活用の改善だが、現状でも機能は動作している。

## 現状

### runtime.rs

`src/runtime.rs:7-9`: `worker_threads(4)` がハードコードされており、
マシンの物理コア数に関わらず常に 4 スレッドで動作する。

### app.rs

`src/app.rs:195-209`: 100ms 固定間隔の `timer` + `done_for_encode` フラグによる polling ループ。
短時間ファイルでは進捗が飛び飛びになり、待機中も 100ms ごとに GPUI スレッドを起こす。

### encode/mod.rs

`src/encode/mod.rs:24-62`: `JobProgress` は `AtomicU64` による共有状態のみを持ち、
進捗更新を UI 側に通知する仕組みを持たない。

### encode/transcode.rs

各エンコード関数 (`src/encode/transcode.rs:352-638`) 内で `progress.add_processed(1)` を呼ぶ。
呼び出し側 (`run_jobs_async`) は `JoinSet` で並列実行し、`encode_file_async` は `tokio::task::spawn_blocking` で同期関数をラップしている。

## 設計方針

### runtime worker_threads の動的決定

`std::thread::available_parallelism()` で物理コア数を取得し `worker_threads` に渡す。
取得失敗時は最小値 (1) にフォールバックする。
`spawn_blocking` 用スレッド数は別途設定しない (tokio のデフォルトに任せる)。

### 進捗イベント駆動化

`JobProgress` に `Arc<tokio::sync::Notify>` フィールドを追加し、
`set_total` / `add_processed` 内で `notify.notify_one()` を呼ぶ。
`app.rs` の polling ループを `notify.notified().await` による待機に置き換える。

## 完了条件

- `worker_threads` が `available_parallelism()` ベースで動的に決定される
- 100ms polling が `Notify` ベースのイベント駆動に置き換わる
- `cargo build` / `cargo clippy --workspace --all-targets -- -D warnings` が 0 warning で完了する

## 解決方法

1. `src/runtime.rs:6-12` の `worker_threads(4)` を `available_parallelism()` による動的決定に変更
2. `src/encode/mod.rs:24-62` の `JobProgress` に `notify: Arc<tokio::sync::Notify>` フィールドを追加
3. `set_total` / `add_processed` 内で `self.notify.notify_one()` を呼ぶ
4. `src/app.rs:174-209` の `progresses` 生成時に `JobProgress` から `Notify` を取得し、polling ループを `notify.notified().await` に置換
5. `src/app.rs:196-209` の `timer(100ms)` + `done_for_encode` フラグ polling を `loop { notify.notified().await; /* UI 更新 */ if done { break; } }` に置換
