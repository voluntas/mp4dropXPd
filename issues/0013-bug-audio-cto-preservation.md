# 入力音声サンプルの composition_time_offset が mux 時に破棄される

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

入力音声サンプルが `composition_time_offset` (CTO) を持っている場合に、mux 時に保持して映像と音声のタイミングを一致させる。

## 優先度根拠

音声と映像の再生タイミングが 1 サンプル単位でずれる。AAC 等の非典型例で問題。

## 現状

`src/encode/transcode.rs:1086` (音声 mux):

```rust
composition_time_offset: None,  // ←入力 CTO を握り潰している
```

`EncodedAudioSample` には `composition_time_offset` フィールドがない。

## 設計方針

`EncodedAudioSample` に `composition_time_offset: Option<i64>` を追加し、入力 CTO を保持して mux に渡す。

## 完了条件

CTO を持つ音声入力で、映像と音声の再生タイミングが一致する (リップシンクの破綻なし)。

## 解決方法

1. `src/encode/transcode.rs:60-64` の `EncodedAudioSample` に `pub composition_time_offset: Option<i64>` フィールドを追加
2. `src/encode/transcode.rs:761-766, 822-828, 838-844` の各音声サンプル生成で `RawSample::composition_time_offset` をコピー
3. `src/encode/transcode.rs:1071-1094` の音声 mux で `composition_time_offset: s.composition_time_offset` に変更

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0013 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0001-0008 との並行**: 本 issue は `EncodedAudioSample` 構造体への 1 フィールド追加で、0001 (`transcode.rs:114-116`)、0002 (`515-540`)、0003 (`306-330`)、0004 (`986-1109`)、0005 (`174-205`)、0006 (`103-106`)、0008 (`src/app.rs`, `src/codec.rs`) とは作業領域が重ならない。`Error::Message` 追加もない。**0007 → 0009 → 0006 → 0001 → 0002 → 0003 → 0004 → 0005 → 0013** の順でコミット可能。
- **0007 で整備される基盤**: `tests/test_encode.rs` への CTO 保持の smoke test 追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### 不具合修正` サブセクション (0009 で確立) に `[FIX] 入力音声サンプルの composition_time_offset を保持` を 1 行で追記する。
