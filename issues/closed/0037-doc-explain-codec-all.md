# AudioCodec::ALL / VideoCodec::ALL の存在意義を明示

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`AudioCodec::ALL` / `VideoCodec::ALL` の意図を doc comment に明記する。

## 優先度根拠

UI のドロップダウン用であることは明白だが、doc に書かれていない。`Default` との重複 (`#[default]` で定義済み) も気になる。

## 現状

`src/codec.rs:9, 28`:

```rust
impl AudioCodec {
    pub const ALL: [Self; 2] = [Self::Aac, Self::Opus];
    ...
}
```

doc comment なし。

## 設計方針

`/// UI のドロップダウン用に全列挙を配列で公開する。` を追加。`From<AudioCodec> for &'static str` を実装して `label()` を trait 経由にすると UI 側の呼び出しが統一できる選択肢もあるが、依存追加を避けるため現状維持で doc のみ追加。

## 完了条件

`AudioCodec::ALL` / `VideoCodec::ALL` の doc comment に意図が記載される。

## 解決方法

`src/codec.rs:9` の `pub const ALL: [Self; 2] = ...` の直前に `/// UI のドロップダウン用に全列挙を配列で公開する。` を追加。`src/codec.rs:28` の `pub const ALL: [Self; 3]` にも同様に追加。

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0037 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0008 との関係**: 0008 (UI 文字列日本語化) と並行可 (`src/codec.rs` の doc comment は UI 文字列ではない)。
- **0026, 0027 との関係**: 0026 (`with_input_bitrates` リネーム) と 0027 (`default_output_path` リファクタ) は `src/codec.rs` の API を変更する。本 issue は doc comment 追加のみで、0026/0027 とは独立。並行可。
- **0007 で整備される基盤**: `tests/test_codec.rs` への `AudioCodec::ALL` / `VideoCodec::ALL` の assert テスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認 (doc comment 変更のみなので影響なし)。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[DOC] AudioCodec::ALL / VideoCodec::ALL の doc comment に UI ドロップダウン用である旨を明記` を 1 行で追記する。
