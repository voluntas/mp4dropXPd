# Mp4FileMuxer の data_offset を手動管理 (ミューザと二重管理)

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-muxer-data-offset-encapsulation
- Polished:
- Reporter:

## 目的

`Mp4FileMuxer` の `data_offset` をミューザ内部に閉じ込め、呼び出し側 (`write_mp4`) での手動管理を解消する。

## 優先度根拠

`data_size` の計算を間違えた場合、シーク位置とミューザの記録がずれるリスク。手動で 2 系統の状態を持つ設計負債。

## 現状

`src/encode/transcode.rs:1043, 1063, 1069, 1087, 1093, 1103-1104`:

```rust
let mut position = initial_bytes.len() as u64;
for (_, kind) in events {
    OutputKind::Video(s) => {
        file.write_all(&s.data)?;
        let data_size = s.data.len();
        ...
        MuxSample {
            ...
            data_offset: position,
            data_size,
        }
        ...
        position += data_size as u64;
    }
    ...
}
let finalized = muxer.finalize()?;
for (offset, bytes) in finalized.offset_and_bytes_pairs() {
    file.seek(SeekFrom::Start(offset))?;
    file.write_all(bytes)?;
}
```

## 設計方針

`shiguredo_mp4` のミューザが `data_offset` を返す API を提供していないか確認する。提供されているならミューザに状態を持たせる。提供されていないなら、`MuxSample::data_offset` 計算を `write_mp4` 専用ヘルパーに閉じ込め、ミューザと `position` の関係を型で表現する。

## 完了条件

`data_offset` の手動管理がなくなる。`MuxSample::data_offset` 設定とファイル書き込みが一体化した API で扱える。

## 解決方法

`src/encode/transcode.rs:986-1109` の `write_mp4` を再設計する。

- `MuxWriter` 構造体を導入し、`write_sample(&mut self, sample_data: &[u8], sample: MuxSample) -> Result<()>` で「ファイル書き込み + data_offset 計算 + muxer.append_sample」を一体化
- `position` の管理を `MuxWriter` 内に private 化
- `finalize()` で muxer からの `offset_and_bytes_pairs` を処理
