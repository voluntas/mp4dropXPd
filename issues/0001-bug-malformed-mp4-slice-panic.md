# 破損 MP4 入力で slice 境界外パニック

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-malformed-mp4-slice-panic
- Polished:
- Reporter:

## 目的

破損 MP4 や `data_offset + data_size > input_data.len()` となる入力に対して、プロセスがパニックせず `Result` で安全にエラーを返すようにする。

## 優先度根拠

ユーザー入力の MP4 ファイルが破損しているケースは普通に発生する。現状の実装ではアプリ全体クラッシュに直結する。

## 現状

`src/encode/transcode.rs:114-116` で `input_data[sample.data_offset as usize..sample.data_offset as usize + sample.data_size].to_vec()` を実行しているが、demuxer の返す `data_offset` / `data_size` の範囲検証を行っていない。

```rust
let data = input_data
    [sample.data_offset as usize..sample.data_offset as usize + sample.data_size]
    .to_vec();
```

`data_offset` は `u64` のため、32bit 環境では `as usize` キャストで truncation の可能性もある。

## 設計方針

3 段階の境界チェックを `Err(Error::Message(...))` で返す。

1. `data_offset.checked_add(data_size as u64)` でオーバーフロー検出
2. `usize::try_from` でキャスト失敗検出
3. ファイルサイズ超過検出

## 完了条件

malformed MP4 を投入してもパニックせず、`Err(Error::Message("invalid sample range: ..."))` 相当が返る。

## 解決方法

`src/encode/transcode.rs:114-116` を以下の手順で書き換える。

- `let end = sample.data_offset.checked_add(sample.data_size as u64).ok_or_else(|| Error::Message("sample data offset overflow".into()))?;`
- `let end = usize::try_from(end).map_err(|_| Error::Message("sample data offset exceeds usize".into()))?;`
- `let start = usize::try_from(sample.data_offset).map_err(|_| Error::Message("sample data offset exceeds usize".into()))?;`
- `if end > input_data.len() { return Err(Error::Message(format!("sample data out of range: {start}..{end}, file size {}", input_data.len()))); }`
- `let data = input_data[start..end].to_vec();`

合わせて `fuzz/fuzz_targets/fuzz_transcode_demux.rs` を追加し、任意バイト列で panic しないことを検証する。
