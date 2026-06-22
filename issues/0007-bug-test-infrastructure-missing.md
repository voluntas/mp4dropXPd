# テスト完全不在 (AGENTS.md / shiguredo-rust 規約違反)

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/add-test-infrastructure
- Polished:
- Reporter:

## 目的

ユニットテスト / PBT / fuzzing のテスト基盤を導入し、公開 API の回帰テストを整備する。

## 優先度根拠

AGENTS.md「コメントはしっかり入れること」「テストはコメントを重視すること」/ shiguredo-rust 規約「PBT (Property-Based Testing) や Fuzzing でテストを行うこと」「unittest は pbt で実現できないものだけを書くこと」「単体テストのファイル名は `tests/test_<module>.rs`」「PBT のファイル名は `pbt/tests/prop_<module>.rs`」すべてに違反。バグの回帰検出が機能していない。

## 現状

`tests/`, `pbt/`, `fuzz/` ディレクトリ皆無。`#[cfg(test)]` ブロック皆無。`cargo test` は `running 0 tests`。`Cargo.toml` に `proptest` 依存なし。

## 設計方針

shiguredo-rust 規約「PBT は proptest を使うこと」「Fuzzing は cargo-fuzz を使うこと」に従い、以下を整備する。

- `Cargo.toml` の `[dev-dependencies]` に `proptest = "1.5"` と `tokio` の `test-util` / `time` features 追加
- `pbt/tests/` に 6 ファイル追加 (`prop_codec.rs`, `prop_input.rs`, `prop_error.rs`, `prop_encode.rs`, `prop_encode_sample_entry.rs`)
- `tests/` に 3 ファイル追加 (`test_input.rs`, `test_encode.rs`, `test_error.rs`)
- `fuzz/fuzz_targets/` に fuzz target 追加
- `.github/workflows/test.yml` で CI 化 (macos-latest のみ、macOS 依存のため)

## 完了条件

- `cargo test` で 0 個以上のテストが実行される
- `cargo test --doc` が成功
- 公開 API (`EncodeRecipe::summary` / `with_input_bitrates`, `is_mp4_file`, `default_output_path`, `sampling_frequency_index`, `build_aac_audio_specific_config`, `average_duration`, `average_bitrate_kbps`, `JobProgress`, `Error::Display`) の PBT が緑
- CI で PR ごとにテストが走る

## 解決方法

shiguredo-rust SKILL.md の「テスト戦略」「カバレッジ駆動のテスト作成手順」に従って段階的に導入する。優先順位:

1. `pbt/tests/prop_codec.rs` (`EncodeRecipe::summary` / `with_input_bitrates`)
2. `pbt/tests/prop_input.rs` (`is_mp4_file` / `filter_mp4_paths`)
3. `pbt/tests/prop_sample_entry.rs` (`sampling_frequency_index` / `build_aac_audio_specific_config`)
4. `pbt/tests/prop_encode.rs` (`average_duration` / `average_bitrate_kbps` / `JobProgress`)
5. `tests/test_input.rs` (拡張子の境界値)
6. `tests/test_error.rs` (Display 全バリアント)
7. `fuzz/fuzz_targets/fuzz_transcode_demux.rs`
8. `.github/workflows/test.yml`

この issue は大きいため、サブタスクとして別 issue に分割するか、複数 PR に分けて段階導入する。
