# expect("shared") メッセージが規約違反の可能性

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0034 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0001-0008 との並行**: 本 issue は `src/app.rs:35` の 1 行メッセージ変更で、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: 新規テスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[FIX] shared 関数の expect メッセージを具体的な mutex poison 説明に` を 1 行で追記する。
