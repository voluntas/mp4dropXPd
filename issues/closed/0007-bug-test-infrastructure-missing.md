# テスト基盤導入 (Cargo.toml 設定 + tests/pbt ディレクトリ + 最初の PBT 1 ファイル)

- Priority: High
- Category: add
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

ユニットテスト / PBT / fuzzing のテスト基盤を導入する。**本 issue は「最小単位の基盤導入」** に限定し、残り PBT ファイル・fuzz target・fixtures 整備は別 issue に切り出す。shiguredo-git 規約「1 コミットは 1 つの論理的な変更にとどめること」「1 issue = 1 コミット」に従い、本 issue のスコープを 1 つの add カテゴリ作業に絞る。

## 優先度根拠

AGENTS.md「コメントはしっかり入れること」「テストはコメントを重視すること」/ shiguredo-rust 規約「PBT (Property-Based Testing) や Fuzzing でテストを行うこと」「unittest は pbt で実現できないものだけを書くこと」「単体テストのファイル名は `tests/test_<module>.rs`」「PBT のファイル名は `pbt/tests/prop_<module>.rs`」すべてに違反。バグの回帰検出が機能していない。

## 現状

`tests/`, `pbt/`, `fuzz/` ディレクトリ皆無。`#[cfg(test)]` ブロック皆無。`cargo test` は `running 0 tests`。`Cargo.toml` に `proptest` 依存なし。

## 設計方針

shiguredo-rust 規約「PBT は proptest を使うこと」「Fuzzing は cargo-fuzz を使うこと」に従うが、**本 issue では PBT は 1 ファイルのみ** (`prop_codec.rs`)、残りの PBT/Unit/Fuzz/Fixtures は別 issue に切り出す。

本 issue のスコープ (最小単位):

1. `Cargo.toml` の `[dev-dependencies]` に `proptest = "1.5"` 追加
2. `pbt/tests/` ディレクトリと `tests/` ディレクトリ作成 (空ファイルで OK)
3. `pbt/tests/prop_codec.rs` 追加: `EncodeRecipe::summary` / `with_input_bitrates` の PBT
4. `.github/workflows/test.yml` 新設: `macos-latest` での `cargo test` 実行

別 issue に切り出す作業 (本 issue のスコープ外):

- `pbt/tests/prop_input.rs` (`is_mp4_file` / `filter_mp4_paths`) → `add-pbt-input`
- `pbt/tests/prop_sample_entry.rs` (`build_mp4a_sample_entry` 経由の間接テスト) → `add-pbt-sample-entry`
- `pbt/tests/prop_encode.rs` (`JobProgress` の決定的な値域テスト) → `add-pbt-encode`
- `tests/test_input.rs` (拡張子の境界値) → `add-unit-input`
- `tests/test_error.rs` (`Error::Display` 全バリアント) → `add-unit-error`
- `tests/test_encode.rs` (smoke test) → 各機能 issue (0001-0006) で追加
- `fuzz/` セットアップ (`fuzz/Cargo.toml` 生成、`cargo fuzz init` 相当) → `add-fuzz-infrastructure`
- `fuzz/fuzz_targets/fuzz_transcode_demux.rs` → `add-fuzz-transcode-demux`
- fixtures ディレクトリ整備 → `add-test-fixtures`

## 完了条件

- `cargo test` で 1 個以上のテストが実行される (`prop_codec.rs` の PBT)
- `.github/workflows/test.yml` が新設され、PR ごとに macos-latest で `cargo test` が走る
- 公開 API (`EncodeRecipe::summary` / `with_input_bitrates`) の PBT が緑
- `cargo clippy --all-targets -- -D warnings` (`prek.toml:28`) を通過する
- [ADD] テスト基盤導入 (`proptest` 依存、`pbt/tests/`、`tests/` ディレクトリ、`.github/workflows/test.yml`、最初の PBT `prop_codec.rs`)
- 別 issue 切り出しの 9 件 (上記) が `issues/` 配下に新規作成されている (もしくは既存の関連 issue に統合)

## 解決方法

### 1. `Cargo.toml` の編集

`Cargo.toml:29` の `[dependencies]` セクションの後に `[dev-dependencies]` セクションを追加:

```toml
[dev-dependencies]
# Property-Based Testing (shiguredo-rust 規約)
proptest = "1.5"
```

### 2. ディレクトリ作成

```bash
mkdir -p pbt/tests tests
```

### 3. `pbt/tests/prop_codec.rs` 追加

`src/codec.rs` の公開 API (`EncodeRecipe::summary`, `with_input_bitrates`) に対する PBT:

```rust
//! `EncodeRecipe` の PBT
//!
//! - `summary`: 映像/音声ビットレートと codec ラベルから決定的な文字列を返す
//! - `with_input_bitrates`: 入力ビットレート (Option<u32>) 2 つを保持して self を返す
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
        let recipe = EncodeRecipe::default().with_input_bitrates(video_kbps, audio_kbps);
        let s1 = recipe.summary();
        let s2 = recipe.summary();
        prop_assert_eq!(s1, s2);
    }

    /// `with_input_bitrates` は `self` を返す (ビルダーパターン)
    #[test]
    fn with_input_bitrates_returns_self(
        video_kbps in proptest::option::of(0u32..100_000),
        audio_kbps in proptest::option::of(0u32..1_000),
    ) {
        let recipe = EncodeRecipe::default();
        let returned = recipe.with_input_bitrates(video_kbps, audio_kbps);
        // ビルドが通れば self が返る (このテストは型チェックも兼ねる)
        let _ = returned.summary();
    }
}
```

### 4. `.github/workflows/test.yml` 新設

```yaml
name: test

on:
  pull_request:
  push:
    branches: [develop]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
      - name: Run tests
        run: cargo test --workspace
      - name: Run clippy
        run: cargo clippy --all-targets -- -D warnings
```

実装時の確認手順:

- **追加で行った作業**:
  - `src/lib.rs` を新規作成し `pub mod codec; pub mod error; pub mod encode; pub mod input;` を宣言した。`pbt/tests/prop_codec.rs` が `mp4dropxpd::codec::EncodeRecipe` をインポートするために必要。
  - `Cargo.toml` に `[[test]]` セクションを追加し、`pbt/tests/prop_codec.rs` をテストターゲットとして登録した。標準の `tests/` ディレクトリと異なり `pbt/tests/` は Cargo が自動認識しないため。
  - `tests/.gitkeep` を追加した (空ディレクトリを git 管理するため)。
- **本 issue の位置づけ**: 0001-0006, 0009 すべてが「0007 と 0009 の双方が closed になるまで着手不可」と依存している最上位 issue。本 issue が完了すると 0001-0006 のテスト追加着手が解禁される。
- **0009 との関係**: 本 issue は `Cargo.toml` 編集のみで `CHANGES.md` 追記を伴う。0009 (`bug-changes-md-missing`) で `CHANGES.md` が新規作成されてから、本 issue の `## 追加` 追記を行う。**0007 → 0009 の順** でコミットする。
- **0001-0006 との関係**: 本 issue 完了後、0001 (`src/encode/transcode.rs:114-116`)、0002 (`transcode.rs:515-540`)、0003 (`transcode.rs:306-330`)、0004 (`transcode.rs:986-1109`)、0005 (`transcode.rs:174-205`)、0006 (`transcode.rs:103-106`) はそれぞれ「`tests/test_encode.rs` への smoke test 追加」を完了条件に含めており、0007 で整備された基盤にテストを書き込む。**0007 → 0009 → 0006 → 0001 → 0002 → 0003 → 0004 → 0005** の順でコミット可能。
- **fuzz の分離**: fuzz (`fuzz/Cargo.toml`, `fuzz_targets/fuzz_transcode_demux.rs`) は別 issue `add-fuzz-infrastructure` / `add-fuzz-transcode-demux` に切り出す。fuzz は nightly ツールチェインが必要なため、本 issue の CI workflow (`test.yml`) には含めない。
- **fixtures の分離**: 0001-0006 が要求するフィクスチャ (破損 MP4 各種、正常 AV1、字幕のみ MP4 等) は別 issue `add-test-fixtures` に切り出す。
- **0032 / 0046 との関係**: 本 issue では `tracing-test` 等のテストユーティリティは追加しない。`tracing::warn!` のテストは 0032 (`add-tracing-logging`) / 0046 (`refactor-remove-tracing-init`) の判断に従う。
- **0033 との関係**: doc test (`cargo test --doc`) は 0033 (`doc-add-spec-references`) のスコープ。本 issue では要求しない。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
