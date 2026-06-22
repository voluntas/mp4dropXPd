# AudioCodec::ALL / VideoCodec::ALL の存在意義を明示

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/doc-explain-codec-all
- Polished:
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
