# f64 ソートで長時間動画で映像/音声の順序破壊

- Priority: Medium
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

mux 時の `f64` ソートを分数 (`Rational { num, den }`) または 100ns 単位の `i128` キーによる整数ソートに置き換える。

## 優先度根拠

90kHz timescale で累積 24 時間で `f64` 精度 (9 桁目以降) を超える差が出る。同着判定が壊れて映像/音声の順序がランダムに入れ替わる。

## 現状

`src/encode/transcode.rs:1034`:

```rust
events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
```

`f64` の `partial_cmp` が `None` を返すのは NaN 同士のみで、ここでは発生しない防御コード。

## 設計方針

events のキーを整数化するため、サンプルベースの priority queue または整数キーの merge sort を採用する。

## 完了条件

24 時間以上の映像/音声を含む MP4 でも、サンプルの順序が崩れない。

## 解決方法

`src/encode/transcode.rs:1002-1034` の `events: Vec<(f64, OutputKind)>` を `events: Vec<MergeEvent { track_kind: TrackKind, timescale: NonZeroU32, timestamp: u64, kind: OutputKind }>` に変更し、最初のサンプルを全トラックから読んでヒープ (`BinaryHeap`) に格納する形にして O(N log N) かつ整数キーの merge sort にする。`sec = timestamp / timescale.get() as f64` ではなく、整数比較で直接順序を決定する。

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0014 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0001-0008 との並行**: 本 issue は `transcode.rs:1002-1034` の `events` Vec と sort ロジックの書き換えで、0001 (`114-116`)、0002 (`515-540`)、0003 (`306-330`)、0004 (`986-1109`)、0005 (`174-205`)、0006 (`103-106`) とは作業領域が重なる可能性がある (0004 は `write_mp4` 内の `events` を使用)。0004 と本 issue の両方が `events` 構造を変える場合、**0004 → 0014 の順** でコミットするか、**0004 と 0014 を統合** して 1 コミットにする。**0007 → 0009 → 0006 → 0001 → 0002 → 0003 → 0004/0014 → 0005** の順。
- **0007 で整備される基盤**: `tests/test_encode.rs` への長時間動画 (24 時間以上) smoke test 追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
