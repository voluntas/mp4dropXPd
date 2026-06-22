# default_output_path のロジックが読みにくい

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-default-output-path
- Polished:
- Reporter:

## 目的

`default_output_path` の文字列整形ロジックを `EncodeRecipe::codec_tag()` メソッドに閉じ込め、可読性を上げる。

## 優先度根拠

`src/encode/output.rs:5-21` は `recipe.video.label().replace('.', "")` と `tag.replace(' ', "")` のように整形が散在し、何をしているのか読みにくい。

## 現状

`src/encode/output.rs:5-21`:

```rust
pub fn default_output_path(input: &Path, recipe: EncodeRecipe) -> PathBuf {
    let stem = input.file_stem().map(|s| s.to_os_string()).unwrap_or_default();
    let parent = input.parent().unwrap_or(Path::new("."));
    let tag = format!(
        "{}_{}",
        recipe.video.label().replace('.', ""),
        recipe.audio.label().to_ascii_lowercase()
    );
    parent.join(format!(
        "{}.{}.mp4",
        stem.to_string_lossy(),
        tag.replace(' ', "")
    ))
}
```

## 設計方針

`EncodeRecipe::codec_tag() -> String` メソッドを `src/codec.rs` に追加し、`H264_Opus` のような整形済み文字列を返すようにする。

## 完了条件

`default_output_path` が `recipe.codec_tag()` を呼ぶだけになる。整形ロジックが `EncodeRecipe` 内に閉じる。

## 解決方法

1. `src/codec.rs` に `impl EncodeRecipe { pub fn codec_tag(&self) -> String { format!("{}_{}", self.video.label().replace('.', ""), self.audio.label().to_ascii_lowercase()).replace(' ', "") } }` を追加
2. `src/encode/output.rs:11-15` を `let tag = recipe.codec_tag();` に置換
