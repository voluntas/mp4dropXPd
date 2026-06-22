# expect("shared") メッセージが規約違反の可能性

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-expect-shared-message
- Polished:
- Reporter:

## 目的

`src/app.rs:34-36` の `expect("shared")` メッセージを具体的にする。

## 優先度根拠

shiguredo-rust スキル SKILL.md:33-35「最低限『このパニックが状況によっては発生する可能性がある』のか、それとも『絶対に発生しない想定（発生した場合は実装バグ）』なのかがメッセージから分かるようにすること」を満たしていない。`"shared"` だけだと何のことか不明。

## 現状

`src/app.rs:34-36`:

```rust
fn shared(cx: &App) -> std::sync::MutexGuard<'_, SharedState> {
    cx.global::<SharedGlobal>().0.lock().expect("shared")
}
```

## 設計方針

メッセージに「mutex poison」であることと「panic 発生時のみ起きうる」ことを明示する。

## 完了条件

`.expect("...")` のメッセージが mutex poison を示す具体的文言になる。

## 解決方法

`src/app.rs:35` を `.expect("SharedGlobal mutex poisoned (mutex を保持するスレッドが panic した場合のみ発生)")` に変更。
