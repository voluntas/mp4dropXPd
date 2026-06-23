//! `EncodeRecipe` の PBT
//!
//! - `summary`: 映像/音声ビットレートと codec ラベルから決定的な文字列を返す
//! - `resolve_input_bitrates`: 入力ビットレート (Option<u32>) 2 つを保持して self を返す
//!
//! 戦略: 各種フィールドをランダム生成し、API 契約 (べき等性、デフォルト動作) を検証する

use mp4dropxpd::codec::EncodeRecipe;
use proptest::prelude::*;

proptest! {
    /// `summary` は入力が同じなら同じ文字列を返す (べき等)
    #[test]
    fn summary_is_idempotent(
        video_kbps in proptest::option::of(0u32..100_000),
        audio_kbps in proptest::option::of(0u32..1_000),
    ) {
        let recipe = EncodeRecipe::default().resolve_input_bitrates(video_kbps, audio_kbps);
        let s1 = recipe.summary();
        let s2 = recipe.summary();
        prop_assert_eq!(s1, s2);
    }

    /// `resolve_input_bitrates` は `self` を返す (ビルダーパターン)
    #[test]
    fn resolve_input_bitrates_returns_self(
        video_kbps in proptest::option::of(0u32..100_000),
        audio_kbps in proptest::option::of(0u32..1_000),
    ) {
        let recipe = EncodeRecipe::default();
        let returned = recipe.resolve_input_bitrates(video_kbps, audio_kbps);
        // ビルドが通れば self が返る (このテストは型チェックも兼ねる)
        let _ = returned.summary();
    }
}
