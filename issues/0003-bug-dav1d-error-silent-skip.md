# dav1d next_frame エラー握り潰し + フラッシュ漏れによる映像音声同期崩壊

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-dav1d-error-silent-skip
- Polished:
- Reporter:

## 目的

AV1 (dav1d) デコード中にエラーが発生した場合に、エラーを呼び出し元に伝播し、最終フレームをフラッシュして取得する。

## 優先度根拠

現状は AV1 入力でフレーム欠落のまま mux フェーズに進み、映像と音声の長さが異なる MP4 が出力される。再生時に同期崩壊・無音・無映像・フリーズを起こす。

## 現状

`src/encode/transcode.rs:306-330` で:

```rust
while let Ok(Some(frame)) = decoder
    .next_frame()
    .map_err(|e| Error::Message(format!("dav1d next_frame error: {e}")))
{
    ...
}
```

`Err` が来た瞬間にループが抜けるだけで、呼び出し元にエラーが伝播しない。ループ脱出後に `decoder.flush()` 相当も呼ばれていないため、最終フレーム群がデコーダバッファに残ったまま捨てられる。

## 設計方針

- `Err` を受けたら `?` で呼び出し元に伝播
- ループ脱出後にデコーダのフラッシュ API を呼び、最終フレームを取り出す

## 完了条件

dav1d がエラーを返した場合に `Err` が呼び出し元 (`encode_video_av1`) まで伝播し、mux まで到達しない。エラーなしで全フレームを処理したときはフラッシュで遅延フレームもすべて取得する。

## 解決方法

`src/encode/transcode.rs:306-330` を `loop { match decoder.next_frame().map_err(|e| Error::Message(format!("dav1d next_frame error: {e}")))? { Some(frame) => { ... } None => break } }` に書き換え、ループ脱出後に dav1d 0.9 系の flush API (もしあれば) を呼び、すべての遅延フレームを処理する。
