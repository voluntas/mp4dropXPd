# 破損 MP4 入力で slice 境界外パニック

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

> **着手条件**: 本 issue は 0007 (テスト基盤整備)、0009 (CHANGES.md 新規作成)、0006 (入力トラック検証) の 3 件がすべて closed になるまで着手不可。

## 目的

`transcode.rs:114-116` の `input_data[sample.data_offset as usize..].to_vec()` に範囲検証がなく、破損 MP4 で `data_offset + data_size > input_data.len()` のときに panic する問題を修正し、`Result` で安全にエラーを返す。

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

demuxer 側 (`shiguredo_mp4` crate の `demux_fmp4_file.rs`、`segment_offset + raw_sample.data_offset` で `data_offset` を合成する箇所) はバッファ末尾との突き合わせを行わない。ファイル末尾を超える `data_offset` / `data_size` の組み合わせが呼び出し側へそのまま渡されるため、本プロジェクト側で防御する必要がある。

本プロジェクトは macOS 専用 (デプロイ対象は 64-bit のみ、`Cargo.toml:35-41` の `[target.'cfg(target_os = "macos")'.dependencies]` 参照)。64-bit macOS では `usize == u64` であり、`usize::try_from` による変換失敗は理論上発生しないが、コードの堅牢性のために `try_from` による検証を維持する。

## 設計方針

`sample.data_offset` と `sample.data_size` から計算されるスライス範囲が `input_data` の範囲内に収まるかを 3 段階で検証する。いずれかの段階で失敗したら `Err` で早期 return する。

1. **加算オーバーフロー検出**: `data_offset.checked_add(data_size as u64)` でオーバーフローを検出する。`data_size` は `usize` だが `end_u64` を得るには `u64` にキャストしてから加算する必要がある
2. **start の `usize` 変換検証**: `usize::try_from(sample.data_offset)` で変換失敗を検出する。64-bit macOS では理論上失敗しないが、コードの堅牢性のために維持する
3. **end の `usize` 変換検証**: `usize::try_from(end_u64)` で変換失敗を検出する。64-bit macOS では理論上失敗しないが、コードの堅牢性のために維持する
4. **ファイルサイズ超過検出**: `end > input_data.len()` で範囲外を検出する。`input_data.len()` は `usize` を返すため、最終比較は両辺 `usize` で行う

ステップ 2 と 3 は 64-bit macOS では到達不能コードだが、`cfg` による条件付きコンパイルよりもコードの簡潔さを優先し、全アーキテクチャ共通のコードパスとする。なお、`checked_add` は非負加算であるため、戻り値 `end_u64` は常に `end_u64 >= sample.data_offset` を満たす。この不変条件により、`usize::try_from` による `start` と `end` の変換後も `start <= end` が成立する。

MP4 コンテナ仕様上、`stsz` ボックスに格納されるサンプルサイズは 32-bit 値 (`u32`)。正当な MP4 から生成される `data_size` は `u32::MAX` を超えない。つまりステップ 1 の `checked_add` オーバーフロー検出とステップ 3 の `usize::try_from(end_u64)` は、正当な入力では実質到達しない防御層である。

検証順序の必然性: `end_u64` 計算で加算オーバーフローが起きる場合、それ以前の `usize` 変換は無意味。そのため `checked_add` を最初に行い、オーバーフローがなければ `usize` 変換、最後に範囲比較の順とする。

エラー表現は当面 `Error::Message(String)` を使う。計 4 箇所 (加算オーバーフロー 1 箇所、`usize` 変換失敗 2 箇所、範囲外 1 箇所) が追加対象。後続の issue 0018 (構造化エラー型 refactor) で、これらのうち範囲外 (`end > input_data.len()`) のケースは `Error::InvalidSampleRange { start, end, file_size }` へ置換する。オーバーフローと `usize` 変換失敗のケースは、0018 側で専用バリアント (`Error::SampleDataOverflow`, `Error::SampleDataConversion`) を新設するか、汎用 `Error::Message` に留めるかを 0018 側で判断する。

## 完了条件

- 破損 MP4 (truncate / `stco` の `chunk_offset` 改竄 / ランダムバイト列) を投入してもプロセスがパニックせず、`Err` で復帰する
- 範囲外 (`end > input_data.len()`) のエラーメッセージに `start` / `end` / `file size` の 3 値が含まれる。加算オーバーフロー、`usize` 変換失敗のケースでは原因種別 (`checked_add` または `try_from`) がエラーメッセージから判別できる。`try_from` 失敗時に start / end のどちらで失敗したかは現状区別できないが、0018 の構造化バリアントで区別される想定
- `tests/test_encode.rs` に以下のテストケースが追加されている:
  - `data_offset + data_size > file_size` (範囲検証が `Err` を返す)
  - `data_offset > file_size` (`start` の時点で範囲外、`Err` を返す)
  - `data_offset` が範囲内かつ `data_size == 0` のときは空スライスが生成され `Ok` になる (境界値。正常系)
  - 加算オーバーフロー (`data_offset = u64::MAX, data_size > 0` で `checked_add` が `None`、`Err` を返す)
- 空ファイル (`input_data.len() == 0`) は demuxer が `read_ftyp_box_header` で `DemuxError::DecodeError` を返すため、本修正のコードパスには到達しない。この前提をテストのコメントに明記する
- 正常 MP4 の smoke test が追加され、修正後も成功することを確認する。正常 MP4 では `data_offset + data_size <= input_data.len()` が常に成立するため、本修正による正常系への副作用はない
- `cargo clippy --workspace --all-targets -- -D warnings` を通過する
- `CHANGES.md` (0009 で新規作成) の `### 不具合修正` に本修正を追記する (`shiguredo-changelog` 規約)

## 解決方法

`src/encode/transcode.rs:114-116` を以下の手順で書き換える。シャドウイングによる clippy 警告を避けるため `end_u64` の中間変数を使う。

```rust
let end_u64 = sample
    .data_offset
    .checked_add(sample.data_size as u64)
    .ok_or_else(|| Error::Message("sample data range overflow".into()))?;
let start = usize::try_from(sample.data_offset)
    .map_err(|_| Error::Message("sample data start exceeds usize".into()))?;
let end = usize::try_from(end_u64)
    .map_err(|_| Error::Message("sample data end exceeds usize".into()))?;
if end > input_data.len() {
    // 範囲外エラーは破損入力の兆候として運用上有用なため warn ログを併用する。
    // 他 3 ケース (オーバーフロー / usize 変換失敗) は内部エラー相当でログ不要。
    tracing::warn!("sample data out of range: {start}..{end}, file size {}", input_data.len());
    return Err(Error::Message(format!(
        "sample data out of range: {start}..{end}, file size {}",
        input_data.len()
    )));
}
let data = input_data[start..end].to_vec();
```

実装時の確認手順:

- **依存順序**: 完了条件のテスト追加と CHANGES.md 追記を満たすには前段 issue が必要。**0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0006 (入力トラック検証) → 0001 (本 issue)** の順で develop に直接コミットする。0007、0009、0006 のすべてが closed になるまで本 issue は着手不可。
- **0007 で整備される基盤**: `tests/test_encode.rs` への本 issue のテスト追加は、0007 が `tests/` の Cargo 設定と fixtures 配置を済ませてから行う。テストファイル作成は 0007 側の責務。本 issue では既存ファイルへのテストケース追加のみ行う。
- **0009 で整備される CHANGES.md**: 完了条件の `### 不具合修正` への追記は、0009 で `CHANGES.md` が新規作成されてから行う。
- **0006 → 0001 の 2 コミット分割**: 0006 は `src/encode/transcode.rs:103-106` のトラック検証、0001 は `src/encode/transcode.rs:111-134` のサンプル範囲検証と、コード上の位置関係 (上位 → 下位) で別領域。`shiguredo-git` 規約「1 コミットは 1 つの論理的な変更にとどめること」に従い、それぞれ 1 コミットずつ develop に直接コミットする。develop 直接コミット運用 (CODEBASE.md) なので PR は作成しない。
- **テスト用フィクスチャの生成**: truncate 済み MP4 は `dd if=normal.mp4 of=truncated.mp4 bs=1 count=N` で生成する。`stco` 改竄版はバイナリエディタで `stco` ボックス内の `chunk_offset` をファイル末尾超過値に書き換えて生成する。ランダムバイト列は `/dev/urandom` から生成する。生成したフィクスチャは `tests/fixtures/` に配置し、テストコードから読み込む。
- **0018 との関係**: 本 issue で導入する 4 箇所の `Error::Message` は 0018 での構造化バリアント置換対象に含める。範囲外の 1 箇所は `Error::InvalidSampleRange` への置換が 0018 に明記されている。残り 3 箇所 (overflow / try_from 変換失敗) は 0018 側で専用バリアント新設または `Error::Message` 据え置きを判断する。
- **同種 slice パニック箇所の調査**: 修正前に `grep -n "as usize" src/` で残存する `as usize` キャストを列挙する。確認対象 (いずれも本 issue のスコープ外):
  - `src/encode/transcode.rs:649-650` (`build_video_samples` の `&raw_samples[i]`) — インデックスパニック
  - `src/encode/transcode.rs:947, 952` (`copy_stride` 内) — `width * height` / `start + width` が `src.len()` 超過でパニック
  - `src/encode/transcode.rs:593, 614` (`enc_frame.pts() as usize`) — `u64 → usize` truncation
  - 本 issue の完了後、必要に応じて `create-issue` スキルで起票する
- **SVT-AV1 pts パスの影響範囲**: `src/encode/transcode.rs:593-597, 614-618` の `samples.get(idx).unwrap_or((0, 1, None))` は panic はしないが不正な timestamp/duration を握り潰す。本 issue のスコープ外。
- **CHANGES.md 追記**: `### 不具合修正` に「破損 MP4 入力時の `data_offset + data_size` 範囲外 panic を `Result` エラーに変換」を 1 行で追記する。
