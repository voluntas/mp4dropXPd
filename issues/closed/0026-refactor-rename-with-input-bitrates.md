# EncodeRecipe::with_input_bitrates の命名が誤解を招く

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0026 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0007 で整備される基盤**: 0007 の `prop_codec.rs` で `with_input_bitrates` をテスト対象としている場合、本 issue 完了後に `resolve_input_bitrates` へのテスト更新が必要。**0007 完了後** に本 issue に着手するか、**0007 と本 issue を統合** して 1 コミットにする。
- **0001-0008 との並行**: 本 issue は `src/codec.rs` の 1 関数リネームで、0001-0008 とは作業領域が重ならない (`transcode.rs` の修正のみ)。並行可。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[REFACTOR] EncodeRecipe::with_input_bitrates を resolve_input_bitrates にリネーム` を 1 行で追記する。
