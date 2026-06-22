# write_mp4 の部分書き込みで出力ファイル破損・元ファイル消失

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-write-mp4-atomic-rename
- Polished:
- Reporter:

## 目的

エンコード失敗時に部分書き込みされた破損 MP4 がそのまま残らないようにする。入力と出力が同じパスを指定した際の元ファイル消失を防ぐ。

## 優先度根拠

`File::create(output)` が既存ファイルを無条件 truncate する。書き込み途中で `?` 演算子で早期 return すると、破損した部分書き込みファイルが残る。次回ユーザーが再エンコードしても上書きできない。入出力同名のケースで元ファイルが失われる。

## 現状

`src/encode/transcode.rs:986-1109` `write_mp4`:

```rust
let mut file = std::fs::File::create(output)?;  // 既存ファイルを truncate
file.write_all(&initial_bytes)?;
...
for (_, kind) in events {
    OutputKind::Video(s) => {
        file.write_all(&s.data)?;  // ? で途中 return
        muxer.append_sample(&sample)
            .map_err(|e| Error::Message(format!("mux append video: {e}")))?;
    }
    ...
}
let finalized = muxer.finalize()
    .map_err(|e| Error::Message(format!("mux finalize: {e}")))?;
```

## 設計方針

一時ファイル (`{output}.mp4.tmp`) に書き、`muxer.finalize()` まで成功してから `rename` でアトミック置換する。失敗時は `Drop` ガードで一時ファイルを削除する。

## 完了条件

エンコード失敗時に破損 MP4 が出力パスに残らない。成功時は今まで通り `output` に MP4 が書き込まれる。途中でクラッシュしても元ファイルが失われない。

## 解決方法

`src/encode/transcode.rs:986-1109` を以下のように修正する。

- 出力パスを `{output}.mp4.tmp` に変更
- `muxer.finalize()` が成功し `file.flush()?;` 完了後に `std::fs::rename(&tmp_path, output)?` でアトミック置換
- 関数内に `Drop` ガード (`struct TempFile(PathBuf); impl Drop for TempFile { ... }`) を導入し、`?` 早期 return 時に一時ファイルを削除
