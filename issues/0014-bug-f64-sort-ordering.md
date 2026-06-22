# f64 ソートで長時間動画で映像/音声の順序破壊

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-f64-sort-ordering
- Polished:
- Reporter:

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
