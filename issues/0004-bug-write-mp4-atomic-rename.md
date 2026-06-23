# write_mp4 の部分書き込みで出力ファイル破損・元ファイル消失

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

エンコード失敗時に部分書き込みされた破損 MP4 がそのまま残らないようにする。入力と出力が同じパスを指定した際の元ファイル消失を防ぐ。

## 優先度根拠

`File::create(output)` (`transcode.rs:995`) が既存ファイルを無条件 truncate する。書き込み途中で `?` 演算子で早期 return すると、破損した部分書き込みファイルが残る。入出力同名のケースで元ファイルが失われる。

## 現状

`src/encode/transcode.rs:986-1109` `write_mp4`:

```rust
let mut file = std::fs::File::create(output)?;  // 既存ファイルを truncate
file.write_all(&initial_bytes)?;
...
for (_, kind) in events {
    OutputKind::Video(s) => {
        file.write_all(&s.data)?;  // ? で途中 return
        ...
    }
    ...
}
let finalized = muxer.finalize()...;
```

呼び出し元は `transcode.rs:170` の 1 箇所のみ。`mod.rs:79-101` の `run_jobs_async` は `tokio::task::JoinSet` で複数ジョブを並列実行するため、同一ディレクトリ内に複数の出力 MP4 が同時に存在しうる。

## 設計方針

一時ファイルに書き、`muxer.finalize()` まで成功してから `rename` でアトミック置換する。失敗時および panic 時は RAII (`Drop`) で一時ファイルを削除する。`std::fs::rename` の atomic 性は **同一ファイルシステム内のみ** なので、tmp 親ディレクトリは `output.parent()` として同一 FS 上の rename を保証する。

一時ファイル名は `{output}.{pid}.{counter}.tmp` 形式 (process id + 単調増加 counter) で衝突回避する。`{output}.mp4.tmp` のような固定名だと `run_jobs_async` の並列実行時に他ジョブの tmp を truncate して破損させる。

- `File::create(&tmp_path)` で既存ファイルの有無に関わらず新規作成 (ただし同名の他ジョブ tmp との衝突は pid + counter で回避)
- 書き込み成功 → `rename(&tmp_path, output)` でアトミック置換
- 失敗 / panic → `TempFile::drop` で `remove_file` (失敗しても警告ログのみ、業務影響なし)
- 入出力同名のケース → 既存ファイル (元ファイル) を `rename` する前に保持したまま、tmp へ書き込み → 成功後に `rename` で置換。元ファイル消失を回避

エラー表現は `From<std::io::Error>` (`error.rs:42-46`) 経由で `Error::Io` に変換される。`Error::Message` を新規追加しないため、0018 (構造化エラー化) への影響なし。0018 着手時に `Error::Rename { from, to }` バリアントの追加余地は残るが、本 issue のスコープ外。

## 完了条件

- エンコード失敗時に破損 MP4 が出力パスに残らない (tmp ファイルが `Drop` で削除される)
- 成功時は今まで通り `output` に MP4 が書き込まれる (`rename` でアトミック置換)
- 途中でパニックしても元ファイルが失われない (RAII で tmp 削除)
- 入力と出力が同じパスのケースで元ファイルが保持される
- 複数ジョブ並列実行時に一時ファイル名が衝突しない
- `tests/test_encode.rs` に正常 MP4 smoke test と、エンコード失敗時に `output` に破損ファイルが残らないことの assert テストが追加されている
- `cargo clippy --workspace --all-targets -- -D warnings` を通過する
- `CHANGES.md` (0009 で新規作成) の `### 不具合修正` に本修正を追記する
- 一時ファイルの `Drop` 失敗時に `tracing::warn!` で英語ログが出力される

## 解決方法

`src/encode/transcode.rs:986-1109` を以下の手順で書き換える。`TempFile` 構造体はファイル内 (関数 `write_mp4` の直前) に private で定義する。

```rust
#[cfg(target_os = "macos")]
/// エンコード失敗時に `Drop` で自動削除される一時ファイル
struct TempFile {
    path: PathBuf,
    committed: bool,
}

#[cfg(target_os = "macos")]
impl TempFile {
    /// `output` と同じディレクトリ内に pid + counter を含む
    /// 一意な一時ファイルパスを生成して `File::create` する
    fn new(output: &Path) -> Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        // プロセス内の他ジョブと衝突しないよう pid + counter を付与
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let pid = std::process::id();
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let parent = output
            .parent()
            .ok_or_else(|| Error::Message("output has no parent directory".into()))?;
        let file_name = output
            .file_name()
            .ok_or_else(|| Error::Message("output has no file name".into()))?
            .to_string_lossy()
            .into_owned();
        let tmp_path = parent.join(format!(".{file_name}.{pid}.{counter}.tmp"));
        // File::create は truncate するため、既存の同名 tmp があれば削除される
        // (pid + counter 衝突時のみ発生、確率は極小)
        std::fs::File::create(&tmp_path)?;
        Ok(Self {
            path: tmp_path,
            committed: false,
        })
    }

    /// 一時ファイルのパスを返す
    fn path(&self) -> &Path {
        &self.path
    }

    /// `rename` 成功後に呼び、`Drop` での削除を防ぐ
    fn commit(mut self) {
        self.committed = true;
    }
}

#[cfg(target_os = "macos")]
impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.committed {
            if let Err(e) = std::fs::remove_file(&self.path) {
                // 失敗時も処理継続 (ログのみ、業務影響なし)
                tracing::warn!(
                    path = %self.path.display(),
                    error = %e,
                    "failed to remove temp file on drop"
                );
            }
        }
    }
}
```

修正後の `write_mp4` の主要変更点:

```rust
#[cfg(target_os = "macos")]
fn write_mp4(
    output: &Path,
    video: Option<VideoOutput>,
    audio: Option<AudioOutput>,
    video_timescale: Option<NonZeroU32>,
) -> Result<()> {
    let mut muxer = Mp4FileMuxer::new().map_err(|e| Error::Message(format!("muxer init: {e}")))?;
    let initial_bytes = muxer.initial_boxes_bytes().to_vec();

    // 既存ファイルを truncate せず、tmp に書き出してから atomic rename する
    let temp = TempFile::new(output)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(temp.path())?;
    file.write_all(&initial_bytes)?;

    // ... (events ループと mux 処理は現状通り) ...

    // finalize
    let finalized = muxer
        .finalize()
        .map_err(|e| Error::Message(format!("mux finalize: {e}")))?;
    for (offset, bytes) in finalized.offset_and_bytes_pairs() {
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(bytes)?;
    }
    file.flush()?;

    // atomic rename で出力先に置換。失敗時は Drop で tmp 削除
    std::fs::rename(temp.path(), output)?;
    temp.commit();
    Ok(())
}
```

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0004 (本 issue)** の順で develop に直接コミットする。
- **0010 との関係**: 本 issue は tmp に書く方針で 0010 の overwrite 制御と独立して動作する。0004 → 0010 の順でコミットすれば conflict しない。
- **0007 で整備される基盤**: `tests/test_encode.rs` へのテスト追加は 0007 完了後に行う。
- **テスト戦略**: 正常系 (完了後に tmp 残骸なし)、失敗系 (破損入力で output にファイル残らない)、入出力同名 (元ファイル保持)、並列実行 (tmp 名衝突なし) の 4 ケース。
- **atomic rename の前提**: `std::fs::rename` は同一 FS 内のみ atomic。`TempFile::new` で `output.parent()` を tmp 親にすることで同一 FS を保証する。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` を通過すること。
- **CHANGES.md 追記**: `### 不具合修正` に「`write_mp4` を tmp + atomic rename 方式に変更」を追記する。
