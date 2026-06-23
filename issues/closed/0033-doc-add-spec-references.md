# 仕様由来コードの根拠資料が未記載

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0033 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **refs/ ディレクトリの扱い**: 0033 で参照する ISO/IEC 14496 シリーズと RFC 7845 の一次資料は本リポジトリに `refs/` ディレクトリが存在しないため、URL のみを doc comment に記載する。`refs/` ディレクトリの整備は別 issue (0037 `doc-explain-codec-all` と統合) で扱う想定。
- **0008 との関係**: 0008 (UI 文字列日本語化) と並行可。`src/encode/sample_entry.rs` の doc comment は UI 文字列ではない (コメント) ため、0016 (英語コメント日本語化) のスコープ。本 issue で doc comment を英語のまま (仕様引用) 追加するか、日本語 (仕様引用 + 日本語説明) で追加するかは判断。**0033 で英語のまま追加し、0016 で日本語化する** のが安全 (0026-0033 の他 doc issue との一貫性)。
- **0016 との関係**: 0016 (英語コメント日本語化) 完了後に本 issue の doc comment が日本語化される可能性がある。順序は 0033 → 0016。
- **0001-0008 との並行**: 本 issue は `src/encode/sample_entry.rs` の doc comment 追加で、0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: doc test (`cargo test --doc`) の追加は 0007 と統合して扱う (0007 完了条件に `cargo test --doc` を含む)。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認 (doc comment 変更のみなので影響なし)。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[DOC] sample_entry.rs の各関数 docstring に ISO/IEC 14496 仕様参照を追加` を 1 行で追記する。
