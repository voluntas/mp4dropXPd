# // finalize 英語コメントの日本語化 (#0016 と統合)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`src/encode/transcode.rs:1098` の `// finalize` 英語コメントを日本語化する。

## 優先度根拠

AGENTS.md「コメントは全て日本語にすること」違反。#0016 (英語コメント) と統合して対応。

## 現状

`src/encode/transcode.rs:1098`:

```rust
// finalize
let finalized = muxer
    .finalize()
    ...
```

## 設計方針

日本語コメントに書き換える。`#0016` の修正でまとめて対応。

## 完了条件

`src/encode/transcode.rs:1098` のコメントが日本語になる。

## 解決方法

#0016 を参照して対応。具体的には `// finalize` を `// muxer をファイナライズする` に置換。

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0016 (本 issue は 0016 と統合)** の順で develop に直接コミットする。0016 (英語コメント日本語化) と本 issue は重複しているため、**0016 に統合** して 0016 内で対応する。0045 は close。
- **0001-0008 との並行**: 0016 と統合後、`src/encode/transcode.rs:1098` の修正は 0001-0008 とは作業領域が重なる可能性がある (0001, 0002, 0003, 0004 が同ファイルを変更)。**0001-0008 → 0016 → 0045 (統合済)** の順でコミットする。
- **0007 で整備される基盤**: 0016 と統合後、新規テスト不要 (コメント変更のみ)。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: 0016 側で `### 不具合修正` サブセクション (0009 で確立) に `[FIX] 英語コメントを日本語に書き換え` を 1 行で追記する (本 issue は 0016 に統合)。
