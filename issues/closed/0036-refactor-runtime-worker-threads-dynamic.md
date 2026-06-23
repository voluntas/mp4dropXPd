# runtime.rs の 4 worker_threads ハードコード

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`src/runtime.rs:8` の `worker_threads(4)` を `available_parallelism()` 等で動的に決定する。

## 優先度根拠

マシンのコア数や macOS の高コア数環境 (M2 Max 等は 12 コア) を考慮していない。`JoinSet` で複数ファイルを並列エンコードする場合はスレッドプール競合が起きる。

## 現状

`src/runtime.rs:6-12`:

```rust
fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
    })
}
```

## 設計方針

`std::thread::available_parallelism()` で物理コア数を取得し、それを `worker_threads` に渡す。`available_parallelism` が失敗した場合は最小値 (1) にフォールバックする。

## 完了条件

`worker_threads` が物理コア数連動になる。`cargo build` / `cargo test` が通る。

## 解決方法

`src/runtime.rs:6-12` を以下のように修正する。

```rust
fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        let worker_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_threads)
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
    })
}
```
