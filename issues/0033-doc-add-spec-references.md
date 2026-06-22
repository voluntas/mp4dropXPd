# 仕様由来コードの根拠資料が未記載

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/doc-add-spec-references
- Polished:
- Reporter:

## 目的

`src/encode/sample_entry.rs` の各 `build_*_sample_entry` 関数に、ISO/IEC 14496 シリーズの節番号と「将来変更される可能性がある」旨を doc comment に明記する。

## 優先度根拠

shiguredo-rust スキル SKILL.md:19「仕様由来の機能を実装する場合は、根拠資料名・節番号・将来変更される可能性があることをコードコメントで明記する」を満たしていない。

## 現状

`src/encode/sample_entry.rs` の関数 docstring は「H.264 (avc1) の SampleEntry を構築する」「H.265 (hev1) の SampleEntry を構築する」のみで、ISO/IEC 14496-15 (MP4 AVC/HEVC) や 14496-3 (AAC) の節番号参照がない。

## 設計方針

各 `build_*_sample_entry` 関数に以下を doc comment に追加する。

- 準拠する仕様書名 (例: ISO/IEC 14496-15)
- 関連する節番号 (例: § 8.3.2.1.2)
- 「将来 box 構造が変更される可能性がある」旨の警告

## 完了条件

`src/encode/sample_entry.rs` の各関数 docstring に仕様参照が含まれる。

## 解決方法

`src/encode/sample_entry.rs` の各関数を以下のように更新する。

- `build_avc1_sample_entry` (line 11) → `/// ISO/IEC 14496-15 § 8.3.2.1.2 (AVC/H.264 SampleEntry) に基づく。仕様変更時は追従要。`
- `build_hev1_sample_entry` (line 38) → `/// ISO/IEC 14496-15 § 8.3.2.1.3 (HEVC/H.265 SampleEntry) に基づく。仕様変更時は追従要。`
- `build_av01_sample_entry` (line 90) → `/// ISO/IEC 14496-15 § 8.3.2.1.4 (AV1 SampleEntry) に基づく。仕様変更時は追従要。`
- `build_mp4a_sample_entry` (line 115) → `/// ISO/IEC 14496-12 § 12.2.2.2 + ISO/IEC 14496-3 (AAC) に基づく。仕様変更時は追従要。`
- `build_opus_sample_entry` (line 145) → `/// RFC 7845 § 5 (OpusSampleEntry) に基づく。仕様変更時は追従要。`
- `build_aac_audio_specific_config` (line 184) → `/// ISO/IEC 14496-3 § 1.6.2.1 (AudioSpecificConfig) に基づく。仕様変更時は追従要。`
