# drain_vt_encoder のタイムアウトなし永久 busy-wait

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-drain-vt-encoder-timeout
- Polished:
- Reporter:

## 目的

VideoToolbox エンコーダが内部遅延・エラー・ドライバ不具合で `Ok(None)` を返し続けた場合に、永久ループせずタイムアウトしてエラーを返すようにする。

## 優先度根拠

`spawn_blocking` のスレッドを永久占有し UI フリーズに直結する。エンコーダ不具合時の可用性問題。

## 現状

`src/encode/transcode.rs:515-540` `drain_vt_encoder` のループ:

```rust
while encoded_frames.len() < expected {
    match encoder.next_frame() {
        Ok(Some(frame)) => { encoded_frames.push(frame); }
        Ok(None) => {
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        Err(e) => { return Err(...); }
    }
}
```

コメントに「期待フレーム数に達するまで無制限に待つ」と明示。以前は `empty_retries > 1000` (合計 100ms) で打ち切っていたが撤廃された。

## 設計方針

連続 `Ok(None)` の累計時間でタイムアウト (例 30 秒) を設ける。`max_frame_delay_count: 1` 設定との整合を取る。

## 完了条件

エンコーダが空フレームを返し続けても 30 秒以内に `Err(Error::Message("encoder stalled"))` が返る。正常完了時は今まで通り全フレームを取得する。

## 解決方法

`src/encode/transcode.rs:515-540` を以下のように修正する。

- `let start = std::time::Instant::now();`
- `let timeout = std::time::Duration::from_secs(30);`
- ループ内で `if start.elapsed() > timeout { return Err(Error::Message("encoder stalled".into())); }` を追加
- タイムアウト値は `const` でモジュール先頭に定義する
