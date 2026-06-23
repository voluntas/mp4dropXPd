# add spec references to sample_entry and clarify UI roles in README

- Priority: Low
- Created: 2026-06-23
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

`src/encode/sample_entry.rs` の各 `build_*_sample_entry` 関数の doc comment に ISO/IEC 14496 仕様参照を追加し、`README.md` にコンテキストメニューと Options ウィンドウの UI 役割分担を明記する。いずれもコード変更を伴わないドキュメントのみの対応。

## 優先度根拠

shiguredo-rust スキルの「仕様由来の機能を実装する場合は、根拠資料名・節番号・将来変更される可能性があることをコードコメントで明記する」を満たしていない。また README の UI 役割分担の説明不足がユーザー混乱の原因になっている。優先度は Low（ドキュメントのみ、機能影響なし）。

## 現状

- `src/encode/sample_entry.rs` (221 行): 各 `build_*_sample_entry` 関数の docstring は「H.264 (avc1) の SampleEntry を構築する」等の簡易な説明のみで、ISO/IEC 14496 シリーズの節番号参照が一切ない
- `README.md` (49 行): コンテキストメニュー (右クリック) と Options ウィンドウの UI 役割分担が明示されておらず、ユーザーが混乱する可能性がある

## 設計方針

### sample_entry.rs の doc comment 追加

各関数の docstring に仕様参照と「仕様変更時は追従要」の警告を追加する。

- `build_avc1_sample_entry` → ISO/IEC 14496-15 § 8.3.2.1.2 (AVC/H.264 SampleEntry)
- `build_hev1_sample_entry` → ISO/IEC 14496-15 § 8.3.2.1.3 (HEVC/H.265 SampleEntry)
- `build_av01_sample_entry` → ISO/IEC 14496-15 § 8.3.2.1.4 (AV1 SampleEntry)
- `build_mp4a_sample_entry` → ISO/IEC 14496-12 § 12.2.2.2 + ISO/IEC 14496-3 (AAC)
- `build_opus_sample_entry` → RFC 7845 § 5 (OpusSampleEntry)
- `build_aac_audio_specific_config` → ISO/IEC 14496-3 § 1.6.2.1 (AudioSpecificConfig)

### README.md の UI 役割分担明記

コンテキストメニューはクイックトグル、Options ウィンドウは詳細設定（ビットレート等）という役割分担を明記する。Auto-encode on drop / Overwrite output は両方から変更可能で即座に反映される旨も追記する。

## 完了条件

- `src/encode/sample_entry.rs` の各関数 docstring に仕様参照（仕様書名・節番号・変更可能性警告）が含まれること
- `README.md` にコンテキストメニューと Options ウィンドウの役割分担が明記されること

## 解決方法

`src/encode/sample_entry.rs` の 6 つの関数 docstring を更新し、`README.md` の機能リストに役割分担の説明を追記する。コード変更なし。
