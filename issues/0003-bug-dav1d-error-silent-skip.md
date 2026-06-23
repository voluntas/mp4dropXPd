# dav1d next_frame エラー握り潰し + ドレイン漏れ

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

AV1 (dav1d) デコード中にエラーが発生した場合に、エラーを呼び出し元に伝播する。エラーなしで全サンプルを処理した場合は、dav1d 内部バッファに残った遅延フレームを `next_frame()` の追加ループで取り出し、最終フレームまで mux に含める。

## 優先度根拠

現状は AV1 入力で `next_frame()` のエラーを握り潰し、フレーム欠落のまま mux フェーズに進む。さらに for ループ脱出後に dav1d 内部バッファの遅延フレームをドレインする処理がない。結果として映像と音声の長さが異なる MP4 が出力され、再生時に同期崩壊を起こす。

## 現状

`src/encode/transcode.rs:306-330` (`decode_video_frames` 内 `Av01` アーム、`#[cfg(target_os = "macos")]` ガード下) で:

```rust
while let Ok(Some(frame)) = decoder
    .next_frame()
    .map_err(|e| Error::Message(format!("dav1d next_frame error: {e}")))
{
    ...
}
```

`Err` が来た瞬間に inner while ループが抜けるだけで、呼び出し元にエラーが伝播しない。外側 `for raw in samples` ループは継続し、破損した dav1d デコーダが後続サンプルを処理し続ける。さらに for ループ脱出後に dav1d 内部バッファに残った遅延フレームを取り出す処理が無く、最終フレーム群が捨てられる。

修正対象の `decode_video_frames` (L270) および呼び出し元 `encode_video_av1` (L542) は `#[cfg(target_os = "macos")]` でガードされており、本バグは macOS 専用パスに限られる。

## 設計方針

`shiguredo_dav1d-2026.1.0` の API に基づく:

- `Decoder::flush()` は内部状態リセット + バッファ内フレーム破棄 API であり、ドレイン目的には使用不可
- `Decoder::finish()` は no-op であり、呼ぶ必要はない
- 公式のドレイン方法は `next_frame()` を `Ok(None)` まで反復呼び出しする
- 既存の `decode_audio_to_pcm` (L700-724、同 `#[cfg(target_os = "macos")]`) のデコードパターン (`?` 伝播 + for 後追加ドレインループ) を踏襲する

具体的な修正方針:

1. inner while ループを `?` 伝播型に書き換え、`Err` を呼び出し元へ即時伝播
2. 外側 `for raw in samples` 脱出後に、`next_frame()` を `Ok(None)` まで反復するドレインループを追加
3. `bit_depth != 8` チェックは inner ループとドレインループの両方で適用

フレーム処理 (`bit_depth` チェック + `copy_stride` + `on_frame`) は inner ループとドレインループで重複するが、クロージャ化は本 issue のスコープ外とする。

エラー表現は当面 `Error::Message(String)` を使う。計 1 箇所 (ドレインループ用) が追加対象。後続の issue 0018 で構造化バリアントへ置換される。本 issue は 0018 の前段にあたり、並行着手はできない。

## 完了条件

- dav1d の `next_frame()` が `Err` を返した場合に `Err` が呼び出し元 (`encode_video_av1`) まで伝播し、mux まで到達しない
- エラーなしで全サンプルを処理したとき、ドレインループで遅延フレームもすべて取得され、最終フレームまで mux に含まれる
- 正常 AV1 MP4 の smoke test が `tests/test_encode.rs` に追加されている
- 破損 AV1 OBU 列入力で `Err` が返ることを assert するテストが追加されている
- `cargo clippy --workspace --all-targets -- -D warnings` を通過する
- `CHANGES.md` (0009 で新規作成) の `### 不具合修正` に本修正を追記する (`shiguredo-changelog` 規約)
- ドレインループ脱出時に `tracing::debug!` で英語ログ (取得フレーム数を含む) が出力される (AGENTS.md「ログメッセージは全て英語にすること」準拠)

## 解決方法

`src/encode/transcode.rs:306-330` を以下のコードで置換する。既存の AudioToolbox デコードパターン (L709-711 の `while let Some(...) = ...?`、L716-724 の後続ドレインループ) とスタイルを統一する。

```rust
SampleEntry::Av01(_) => {
    let mut decoder =
        shiguredo_dav1d::Decoder::new(shiguredo_dav1d::DecoderConfig::default())
            .map_err(|e| Error::Message(format!("dav1d init error: {e}")))?;
    for raw in samples {
        decoder
            .decode(&raw.data)
            .map_err(|e| Error::Message(format!("dav1d decode error: {e}")))?;
        // `?` で Err を呼び出し元へ即時伝播する。
        // 破損 dav1d デコーダで後続サンプルを処理し続けるリスクを排除するため。
        while let Some(frame) = decoder
            .next_frame()
            .map_err(|e| Error::Message(format!("dav1d next_frame error: {e}")))?
        {
            if frame.bit_depth() != 8 {
                return Err(Error::Message(
                    "AV1 デコード結果が 8-bit ではありません (現状は 8-bit I420 のみ対応)"
                        .into(),
                ));
            }
            let y = copy_stride(frame.y_plane(), frame.y_stride(), w, h);
            let u = copy_stride(frame.u_plane(), frame.u_stride(), w / 2, h / 2);
            let v = copy_stride(frame.v_plane(), frame.v_stride(), w / 2, h / 2);
            on_frame(&y, &u, &v)?;
        }
    }
    // 全サンプル処理後、dav1d 内部バッファに残った遅延フレームを
    // `next_frame()` を `Ok(None)` まで反復呼び出ししてドレインする。
    // `Decoder::flush()` は内部状態リセット + バッファ破棄 API のため使用不可。
    // `Decoder::finish()` は no-op。
    // AudioToolbox デコードパターン (transcode.rs:716-724) を踏襲。
    let mut drained = 0usize;
    while let Some(frame) = decoder
        .next_frame()
        .map_err(|e| Error::Message(format!("dav1d drain error: {e}")))?
    {
        if frame.bit_depth() != 8 {
            return Err(Error::Message(
                "AV1 デコード結果が 8-bit ではありません (現状は 8-bit I420 のみ対応)"
                    .into(),
            ));
        }
        let y = copy_stride(frame.y_plane(), frame.y_stride(), w, h);
        let u = copy_stride(frame.u_plane(), frame.u_stride(), w / 2, h / 2);
        let v = copy_stride(frame.v_plane(), frame.v_stride(), w / 2, h / 2);
        on_frame(&y, &u, &v)?;
        drained += 1;
    }
    tracing::debug!("dav1d drain complete: {} delayed frames", drained);
    Ok(())
}
```

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0003 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0007 で整備される基盤**: `tests/test_encode.rs` へのテスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **テストフィクスチャ**: 破損 AV1 OBU 列のフィクスチャは 0007 で整備される fixtures ディレクトリに配置する。正常 AV1 入力は `ffmpeg -c:v libaom-av1` 等で生成可能。
- **dav1d 破損 OBU 後のデコーダ状態**: `Err` 即時 return により後続サンプル処理はスキップされる。`decoder` は `encode_video_av1` 関数のスコープを抜けて `Drop` される運用を前提とし、明示的なデコーダリセットは行わない。
- **dav1d ドレインの無限ループリスク**: ドレイン用 `while let Some` は `Ok(None)` で正常終了する。`Ok(Some)` を返し続けるケースは 0002 の `drain_vt_encoder` タイムアウト機構に倣った別 issue で対応する。
- **0018 との関係**: 本 issue で追加する `Error::Message` 1 箇所は 0018 の refactor 対象に含まれる。本 issue は 0018 の前段であり、並行着手はできない。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` を通過すること。
- **CHANGES.md 追記**: `### 不具合修正` に「AV1 (dav1d) デコードのエラー握り潰しと最終フレームドレイン漏れを修正」を 1 行で追記する。
