# EncodeRecipe 周辺の小規模リファクタリング統合 (リネーム + codec_tag + DEFAULT_VIDEO_TIMESCALE)

- Priority: Low
- Created: 2026-06-23
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

src/codec.rs、src/encode/output.rs、src/encode/transcode.rs に散在する小規模リファクタリング 3 件を 1 件にまとめて実施する。いずれも機能変更を伴わないコード整理。元 issue: #0026, #0027, #0035。

## 優先度根拠

機能変更を伴わない軽微なリファクタリングであり、緊急性は低い。

## 現状

1. `EncodeRecipe::with_input_bitrates()` (`src/codec.rs:56`) — `with_*` は通常不変な変換を意味するが、このメソッドは `self` の未設定フィールドを入力ビットレートで上書きする。「入力ビットレートを解決する」という意味を名前に反映できていない
2. `default_output_path()` (`src/encode/output.rs:5-21`) — codec tag の整形ロジック (`video.label().replace('.', "")` / `tag.replace(' ', "")`) が output.rs に埋まっており、EncodeRecipe の責務として適切でない
3. `NonZeroU32::new(30).expect("30 != 0")` (`src/encode/transcode.rs:361, 443, 552`) — デフォルト映像 timescale (30 fps) を示すマジックナンバーが 3 箇所にハードコードされている

## 設計方針

機能の変更は一切行わず、コードの整理のみを行う。
`CODEBASE.md` の規約に従い develop に直接コミットする (Branch フィールドの値は実装着手時に使う想定の名前)。

## 完了条件

- `EncodeRecipe::with_input_bitrates()` が `resolve_input_bitrates()` にリネームされ、`src/encode/transcode.rs:150` の呼び出し元が更新されていること
- `EncodeRecipe::codec_tag() -> String` が追加され、`src/encode/output.rs` の `default_output_path()` がそれを利用する形に変更されていること
- `src/encode/transcode.rs` に `const DEFAULT_VIDEO_TIMESCALE: NonZeroU32` が定義され、3 箇所のハードコードが置き換えられていること
- `cargo build`、`cargo test`、`cargo clippy --workspace --all-targets -- -D warnings` がすべてパスすること

## 解決方法

### 1. src/codec.rs — リネーム + codec_tag() 追加

```rust
// リネーム
pub fn resolve_input_bitrates(
    self,
    video_input_kbps: Option<u32>,
    audio_input_kbps: Option<u32>,
) -> Self {
    // 実装は現行のまま
}

// 新規追加
pub fn codec_tag(&self) -> String {
    format!(
        "{}_{}",
        self.video.label().replace('.', ""),
        self.audio.label().to_ascii_lowercase()
    )
    .replace(' ', "")
}
```

### 2. src/encode/output.rs — codec_tag() 利用

`default_output_path()` 内の `tag` 変数の整形ロジック (L11-15) を `let tag = recipe.codec_tag();` に置き換え、後続の `tag.replace(' ', "")` (L19) を削除する。

### 3. src/encode/transcode.rs — 呼び出し更新 + const 定義

- L150: `recipe.with_input_bitrates(...)` → `recipe.resolve_input_bitrates(...)`
- `use` 群の直後に `const DEFAULT_VIDEO_TIMESCALE: NonZeroU32 = NonZeroU32::new(30).expect("30 != 0");` を追加
- L361, 443, 552: `NonZeroU32::new(30).expect("30 != 0")` → `DEFAULT_VIDEO_TIMESCALE`
