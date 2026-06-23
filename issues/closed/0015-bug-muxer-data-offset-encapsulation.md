# Mp4FileMuxer の data_offset を手動管理 (ミューザと二重管理)

- Priority: Medium
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

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

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0015 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0004 との並行**: 0004 (`TempFile` 導入) と本 issue は `write_mp4` (`transcode.rs:986-1109`) を変更する。**0004 → 0015 の順** でコミットするか、**0004 と 0015 を統合** して 1 コミットにする。
- **0007 で整備される基盤**: `tests/test_encode.rs` への smoke test 追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
