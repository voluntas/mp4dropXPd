# 破損 MP4 入力で slice 境界外パニック

- Priority: High
- Created: 2026-06-22
- Completed: 2026-06-23

- Branch: なし (CODEBASE.md 参照。develop に直接コミット)
- Polished: 2026-06-23

> **着手条件**: 本 issue は 0007 (テスト基盤整備)、0009 (CHANGES.md 新規作成)、0006 (入力トラック検証) の 3 件がすべて closed になるまで着手不可。

## 目的

`transcode.rs:114-116` の `input_data` のスライス (`.to_vec()`) に範囲検証がなく、破損 MP4 で `data_offset + data_size > input_data.len()` のときに panic する問題を修正し、`Result` で安全にエラーを返す。

## 優先度根拠

ユーザー入力の MP4 ファイルが破損しているケースは普通に発生する。現状の実装 (`src/encode/transcode.rs:114-116`) ではアプリ全体クラッシュに直結する。`shiguredo_mp4` の `demux::Sample` は `data_offset: u64` と `data_size: usize` を提供するが、demuxer はファイル末尾との突き合わせ検証を行わない。呼び出し側で防御的に範囲検証するのは本プロジェクトの責務範囲。

## 現状

`src/encode/transcode.rs:111-116` で `demuxer.next_sample()` から得た `demux::Sample.data_offset: u64` / `data_size: usize` (`shiguredo_mp4` crate の `demux` モジュールで定義) をそのまま `Vec<u8>` 型の `input_data` のインデックスとして使い、範囲検証が一切ない。

```rust
// transcode.rs:114-116
let data = input_data
    [sample.data_offset as usize..sample.data_offset as usize + sample.data_size]
    .to_vec();
```

demuxer 側 (`shiguredo_mp4` crate の demuxer) はバッファ末尾との突き合わせを行わない。

本プロジェクトは macOS 専用 (デプロイ対象は 64-bit のみ、`Cargo.toml:35-41` の `[target.'cfg(target_os = "macos")'.dependencies]` 参照)。64-bit macOS では `usize == u64` であり、`usize::try_from` による変換失敗は理論上発生しないが、コードの堅牢性のために `try_from` による検証を維持する。

## 設計方針

`sample.data_offset` と `sample.data_size` から計算されるスライス範囲が `input_data` の範囲内に収まるかを以下の全 4 項目で検証する。いずれかの項目で失敗したら `Err` で早期 return する。

1. **加算オーバーフロー検出**: `data_offset.checked_add(data_size as u64)` でオーバーフローを検出する。`data_size` は `usize` だが `end_u64` を得るには `u64` にキャストしてから加算する必要がある
2. **start の `usize` 変換検証**: `usize::try_from(sample.data_offset)` で変換失敗を検出する
3. **end の `usize` 変換検証**: `usize::try_from(end_u64)` で変換失敗を検出する
4. **ファイルサイズ超過検出**: `end > input_data.len()` で範囲外を検出する。`input_data.len()` は `usize` を返すため、最終比較は両辺 `usize` で行う

ステップ 2 と 3 は 64-bit macOS では `usize == u64` のため到達不能コードだが、`cfg` による条件付きコンパイルよりもコードの簡潔さを優先し、全アーキテクチャ共通のコードパスとする。なお、`checked_add` は非負加算であるため、戻り値 `end_u64` は常に `end_u64 >= sample.data_offset` を満たす。この不変条件により、`usize::try_from` による `start` と `end` の変換後も `start <= end` が成立する。

検証順序の必然性: `end_u64` 計算で加算オーバーフローが起きる場合、それ以前の `usize` 変換は無意味。そのため `checked_add` を最初に行い、オーバーフローがなければ `usize` 変換、最後に範囲比較の順とする。なお正当な MP4 では `data_size <= u32::MAX` のため `checked_add` と `try_from(end_u64)` は実質到達しない防御層である。

エラー表現は当面 `Error::Message(String)` を使う。既存コードベースではドメインロジック由来エラーは日本語、外部エラーラップ (`"demux error: {e}"` 等) は英語が慣例。本 issue で追加する 4 箇所はいずれもドメインロジック由来のため日本語で記述する。計 4 箇所 (加算オーバーフロー 1 箇所、`usize` 変換失敗 2 箇所、範囲外 1 箇所) が追加対象。ただし `tracing::warn!` のログメッセージは AGENTS.md 規約により英語とする。後続の issue 0018 (構造化エラー型 refactor) で、これらのうち範囲外 (`end > input_data.len()`) のケースは `Error::InvalidSampleRange { start, end, file_size }` へ置換する。オーバーフローと `usize` 変換失敗のケースは、0018 側で専用バリアント (`Error::SampleDataOverflow`, `Error::SampleDataConversion`) を新設するか、汎用 `Error::Message` に留めるかを 0018 側で判断する。

## 完了条件

- 破損 MP4 (truncate / `stco`/`co64` の `chunk_offset` 改竄 / ランダムバイト列) を投入してもプロセスがパニックせず、`Err` で復帰する
- 範囲外 (`end > input_data.len()`) のエラーメッセージに `start` / `end` / `file size` の 3 値が含まれる。加算オーバーフロー、`usize` 変換失敗のケースでは原因種別 (`checked_add` または `try_from`) がエラーメッセージから判別できる。`try_from` 失敗時はエラーメッセージの文言によって start / end のどちらで失敗したか区別可能 (構造化バリアントではないためプログラム上のマッチには不向き。0018 で改善予定)
- (0007 で `tests/` 基盤が整備された前提で) `tests/test_encode.rs` に以下のテストケースが追加されている:
  - `data_offset + data_size > file_size` (範囲検証が `Err` を返す)
  - `data_offset > file_size` (`start` の時点で範囲外、`Err` を返す)
  - `data_offset` が範囲内かつ `data_size == 0` のときは空スライスが生成され `Ok` になる (境界値。正常系)
  - 加算オーバーフロー (`data_offset = u64::MAX, data_size > 0` で `checked_add` が `None`、`Err` を返す)。`co64` ボックスを含む MP4 フィクスチャが必要であり、`tests/fixtures/` に専用のテスト用ファイルを配置する
- `try_from` 検証 (項目 2, 3) は macOS 64-bit では `usize == u64` のため到達不能。コード上の防御層として存在するが、単体テストは物理的に不可能なためテスト対象外とする。この設計判断をコメントに明記する
- 空ファイル (`input_data.len() == 0`) は demuxer がエラーを返すため、本修正のコードパスには到達しない。テストは不要だが、前提としてコメントに明記する
- 正常 MP4 (H.264+AAC 1 ファイルを smoke test の最低限とする) のテストが追加され、修正後も成功することを確認する。正常 MP4 では `data_offset + data_size <= input_data.len()` が常に成立するため、本修正による正常系への副作用はない
- `cargo clippy --all-targets -- -D warnings` を通過する
- `cargo test --workspace --all-targets` が全テスト成功で終了する
- `CHANGES.md` (0009 で新規作成) の `### 不具合修正` に `[FIX] 破損 MP4 入力時の data_offset + data_size 範囲外 panic を Result エラーに変換する` を追記する (`shiguredo-changelog` 規約)

## 解決方法

`src/encode/transcode.rs:114-116` を以下の手順で書き換える。シャドウイングによる clippy 警告を避けるため `end_u64` の中間変数を使う。

```rust
let end_u64 = sample
    .data_offset
    .checked_add(sample.data_size as u64)
    .ok_or_else(|| Error::Message("サンプルデータ範囲がオーバーフローしました".into()))?;
let start = usize::try_from(sample.data_offset)
    .map_err(|_| Error::Message("サンプルデータ開始位置が usize を超えました".into()))?;
let end = usize::try_from(end_u64)
    .map_err(|_| Error::Message("サンプルデータ終了位置が usize を超えました".into()))?;
if end > input_data.len() {
    // 破損入力の兆候として warn ログを出力する
    tracing::warn!("sample data out of range: {start}..{end}, file size {}", input_data.len());
    return Err(Error::Message(format!(
        "サンプルデータが範囲外です: {start}..{end}, ファイルサイズ {}",
        input_data.len()
    )));
}
let data = input_data[start..end].to_vec();
```

実装時の確認手順:

- **依存順序**: 完了条件のテスト追加と CHANGES.md 追記を満たすには前段 issue が必要。**0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0006 (入力トラック検証) → 0001 (本 issue)** の順で develop に直接コミットする。
- **0007 で整備される基盤**: `tests/test_encode.rs` への本 issue のテスト追加は、0007 が `tests/` の Cargo 設定と fixtures 配置を済ませてから行う。テストファイル作成は 0007 側の責務。本 issue では既存ファイルへのテストケース追加のみ行う。
- **0009 で整備される CHANGES.md**: 完了条件の `### 不具合修正` への追記は、0009 で `CHANGES.md` が新規作成されてから行う。
- **0006 → 0001 の 2 コミット分割**: 0006 は `src/encode/transcode.rs:103-106` に映像/音声トラック不在時の早期エラー検出を追加する。0001 は `src/encode/transcode.rs:114-116` のサンプル範囲検証を追加する。両者はコード上の近接領域にあり、0006 が先に完了することで 0001 のテストが落ち着いたコードベースに対して書ける。`shiguredo-git` 規約「1 コミットは 1 つの論理的な変更にとどめること」に従い、それぞれ 1 コミットずつ develop に直接コミットする。develop 直接コミット運用 (CODEBASE.md) なので PR は作成しない。
- **テスト用フィクスチャの生成**: truncate 済み MP4 (最初のサンプル途中でファイル切断)、`stco`/`co64` 改竄版 (`chunk_offset` をファイル末尾超過値に書き換え)、ランダムバイト列の 3 種を `tests/fixtures/` に配置する。改竄は決定論的なスクリプトで生成し再現性を確保する。
- **エラー発生時の既存サンプル破棄**: 範囲外エラー発生時に `return Err(...)` で `transcode` 関数全体を脱出する。それまでに正常に蓄積された `video_samples` / `audio_samples` はすべて破棄される。部分的な出力は生成しない。これは「panic させない」ための最小限の修正として妥当であり、破損ファイルの部分救済は本 issue のスコープ外。
- **PBT 戦略**: 本修正の核心は境界値検証であり、proptest による PBT は後続 issue で導入する (0007 で基盤導入済みの proptest を活用)。本 issue では単体テスト 4 ケース (範囲切れ 2 + 境界値 1 + オーバーフロー 1) + smoke test 1 ファイルで十分。
- **0018 との関係**: 本 issue で導入する 4 箇所の `Error::Message` は 0018 の構造化バリアント置換対象。
- **同種 slice パニック箇所の調査**: 本修正では `src/encode/transcode.rs:114-116` のみを対象とする。他箇所の類似の潜在パニック (インデックス境界、`copy_stride` の slice 範囲外、`as usize` truncation) は別途 `create-issue` スキルで起票して対応する。
- **SVT-AV1 pts パスの影響範囲**: `src/encode/transcode.rs` の `samples.get(idx).unwrap_or((0, 1, None))` は別課題 (本 issue スコープ外)。
- **lib.rs の確認**: 0007 完了時に `src/lib.rs` が `pub mod encode;` と `pub mod error;` を公開していることを確認する。テストからのインポートに必要。
