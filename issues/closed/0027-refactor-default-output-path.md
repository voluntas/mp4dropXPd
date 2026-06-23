# default_output_path のロジックが読みにくい

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0027 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0008 との関係**: 0008 (UI 文字列日本語化) は `src/encode/output.rs` を含まない。本 issue とは独立。並行可。
- **0001-0008 との並行**: 本 issue は `src/codec.rs` と `src/encode/output.rs` のリファクタで、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: 0007 の `pbt/tests/prop_codec.rs` で `default_output_path` のテストを追加する場合、本 issue 完了後の API を対象に。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] default_output_path の整形ロジックを EncodeRecipe::codec_tag() に移動` を 1 行で追記する。
