# EncodeRecipe::with_input_bitrates の命名が誤解を招く

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-rename-with-input-bitrates
- Polished:
- Reporter:

## 目的

`with_input_bitrates` を意味の分かる名前にリネームする。

## 優先度根拠

`with_*` は通常「不変な変換 (新しい値を返す)」を意味するが、この実装は `self` の一部を破壊的に上書きしている (codec.rs:56-74)。命名から挙動が読み取れない。

## 現状

`src/codec.rs:56-74`:

```rust
pub fn with_input_bitrates(
    self,
    video_input_kbps: Option<u32>,
    audio_input_kbps: Option<u32>,
) -> Self {
    Self {
        video_bitrate_kbps: if self.video_bitrate_kbps == 0 {
            video_input_kbps.unwrap_or(self.video_bitrate_kbps)
        } else {
            self.video_bitrate_kbps
        },
        audio_bitrate_kbps: if self.audio_bitrate_kbps == 0 {
            audio_input_kbps.unwrap_or(self.audio_bitrate_kbps)
        } else {
            self.audio_bitrate_kbps
        },
        ..self
    }
}
```

`self` を受け取り、新しい `Self` を返すが、構造体更新構文で `self` の一部を上書きしている。

## 設計方針

`fn resolve_input_bitrates(self, video: Option<u32>, audio: Option<u32>) -> Self` にリネームする。`resolve` は「未確定値を入力で確定する」ニュアンス。

## 完了条件

関数名が `resolve_input_bitrates` に変更される。呼び出し元 (`src/encode/transcode.rs:150`) も合わせて更新される。

## 解決方法

`src/codec.rs:56` を `pub fn resolve_input_bitrates(self, video_input_kbps: Option<u32>, audio_input_kbps: Option<u32>) -> Self` に変更。`src/encode/transcode.rs:150` の呼び出し元も `recipe.resolve_input_bitrates(...)` に更新。
