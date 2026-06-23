# drain_vt_encoder のタイムアウトなし無限ポーリングループ

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

VideoToolbox エンコーダが内部遅延で `Ok(None)` を返し続けた場合に、永久ループせずタイムアウトしてエラーを返すようにする。

## 優先度根拠

`tokio::task::spawn_blocking` (`mod.rs:74`) のスレッドを永久占有し、エンコード処理が永久ハングする。`Ok(None)` が連続するのは VideoToolbox エンコーダの内部状態不整合や macOS のリソース枯渇時に稀に発生し、一度発生すると同一プロセスでは同一ジョブを中断できず、別ジョブの進捗も停止する。`shiguredo-rust` 規約「性能より堅牢性を優先」に従い、`Result` で安全にエラーを返す経路を設ける。

## 現状

`src/encode/transcode.rs:515-540` `drain_vt_encoder` のループ:

```rust
let mut encoded_frames: Vec<shiguredo_video_toolbox::EncodedFrame> = Vec::new();
while encoded_frames.len() < expected {
    match encoder.next_frame() {
        Ok(Some(frame)) => {
            encoded_frames.push(frame);
        }
        Ok(None) => {
            // フレームがまだ届いていないだけなので短いスリープで待つ
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
        Err(e) => {
            return Err(Error::Message(format!("video next_frame error: {e}")));
        }
    }
}
```

`Err(e)` パスは既に関数から `return` するため本 issue のスコープ外。`Ok(None)` パスだけがストール検知の対象。

呼び出し元は `transcode.rs:409` (H.264) と `transcode.rs:490` (H.265) の 2 箇所のみ。SVT-AV1 (`transcode.rs:592, 613`) は `Ok(None)` を内部消費する設計 (内部ブロッキング読み) のため本 issue のスコープ外。dav1d decode (`transcode.rs:314-316`)、AudioToolbox decode/encode (`transcode.rs:709-724, 759-767`) も同様。

## 設計方針

連続 `Ok(None)` の累計時間でタイムアウトを設ける。`Ok(Some(frame))` 受信時にリセットする「連続」方式であり、`start.elapsed() > timeout` のような「通算の wall-clock 経過時間」ではない。VideoToolbox エンコーダは `max_frame_delay_count: 1` (transcode.rs:385, 467) でフレーム遅延 1 に制限されており、正常時は単一フレームあたり 50〜200ms 程度で `Ok(Some)` を返す。**30 秒**は単一フレームエンコードが想定の 150〜600 倍遅れた状態をストールとみなす閾値。正常な長尺動画 (1 時間 30fps = 10 万フレーム) でもフレーム間隔は 33ms であり、1 回の `Ok(None)` が 30 秒連続することはない。Intel Mac の低スペック環境や高負荷時のゆらぎを考慮しても 30 秒は十分に安全なマージンを持つ。

- 連続 `Ok(None)` 累計時間で 30 秒経過 → `Err(Error::Message(...))` を返す
- `Ok(Some)` 受信時 → タイマーをリセットして継続
- `Err(e)` 受信時 → 既存通り `Err` を `?` で伝播
- `expected == 0` → ループに入らず即 `Ok(vec![])` を返す (既存挙動と同じ)

`last_progress` は関数エントリ時に `Instant::now()` で初期化する。これにより、最初の `next_frame()` が `Ok(None)` だった場合の計測起点は関数エントリ時となる。厳密には「最初の Ok(None) 観測時点」ではないが、エントリから最初の `next_frame()` まではマイクロ秒オーダーであり、30 秒のタイムアウトに対して誤差は無視できる。

エラー表現は当面 `Error::Message(String)` を使う。計 1 箇所が追加対象。後続の issue 0018 (構造化エラー型 refactor) の完了後に `Error::DrainTimeout { elapsed, expected, received }` へ置換される想定。本 issue で導入する `Error::Message` は 0018 の `Error::Message` 使用箇所削減目標 (47 → 10 箇所以下) の対象であり、0018 の設計方針に記載済み。

## 完了条件

- エンコーダが `Ok(None)` を返し続けても連続 30 秒以内に `Err` で復帰する
- 正常完了時は今まで通り全フレームを取得する
- `Ok(Some)` 受信時にタイムアウトタイマーがリセットされる
- `Err` メッセージに経過時間 (`elapsed`)、期待フレーム数 (`expected`)、取得済みフレーム数 (`received`) の 3 値が含まれる
- 正常 MP4 の smoke test が `tests/test_encode.rs` に追加され、修正後も成功することを確認する。smoke test の実行時間は 30 秒より十分短いこと。CI 環境での実測値を smoke test のコメントに記載する
- タイムアウト発生時の挙動テスト: `#[cfg(test)]` で `DRAIN_VT_ENCODER_STALL_TIMEOUT` を短縮 (例: `Duration::from_millis(200)`) する方法を利用するか、正常系 smoke test + CI での手動確認で妥協する。モック禁止規約 (AGENTS.md) 下では偽の `VtEncoder` を作れないため、タイムアウトパスの自動テストは必須としない
- `cargo clippy --workspace --all-targets -- -D warnings` を通過する
- `CHANGES.md` (0009 で新規作成) の `### 不具合修正` に本修正を追記する (`shiguredo-changelog` 規約)

## 解決方法

`src/encode/transcode.rs:515-540` を以下の手順で書き換える。`const` は `drain_vt_encoder` の直前 (line 515 直前) に private で配置する。`#[cfg(target_os = "macos")]` ガードは const と関数本体の両方に必要 (非 macOS ターゲットでは `VtEncoder` 型がスコープに入らないため)。

```rust
#[cfg(target_os = "macos")]
/// `drain_vt_encoder` が `Ok(None)` を連続して受け取ったまま
/// 待機してよい最大の待ち時間
const DRAIN_VT_ENCODER_STALL_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(30);
```

修正後の `drain_vt_encoder` 本体 (既存の 520-523 行のコメントは本コードで置換される):

```rust
#[cfg(target_os = "macos")]
fn drain_vt_encoder(
    encoder: &mut VtEncoder,
    expected: usize,
) -> Result<Vec<shiguredo_video_toolbox::EncodedFrame>> {
    let mut encoded_frames: Vec<shiguredo_video_toolbox::EncodedFrame> = Vec::new();
    let mut last_progress = std::time::Instant::now();
    while encoded_frames.len() < expected {
        match encoder.next_frame() {
            Ok(Some(frame)) => {
                // フレームが届いた → ストール判定をリセット
                last_progress = std::time::Instant::now();
                encoded_frames.push(frame);
            }
            Ok(None) => {
                // フレームがまだ届いていない。連続ストール判定。
                let elapsed = last_progress.elapsed();
                if elapsed >= DRAIN_VT_ENCODER_STALL_TIMEOUT {
                    let received = encoded_frames.len();
                    tracing::warn!(
                        elapsed = ?elapsed,
                        expected,
                        received,
                        "video encoder stalled"
                    );
                    return Err(Error::Message(format!(
                        "video encoder stalled: elapsed {elapsed:?}, expected {expected} frames, received {received}"
                    )));
                }
                // スリープで CPU を譲りつつポーリング間隔を確保
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
            Err(e) => {
                return Err(Error::Message(format!("video next_frame error: {e}")));
            }
        }
    }
    Ok(encoded_frames)
}
```

実装時の確認手順:

- **依存順序**: 完了条件のテスト追加と CHANGES.md 追記を満たすには前段 issue が必要。**0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0002 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0007 で整備される基盤**: `tests/test_encode.rs` への本 issue のテスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **0009 で整備される CHANGES.md**: 完了条件の `### 不具合修正` への追記は、0009 で `CHANGES.md` が新規作成されてから行う。
- **0011 との連携**: `0011-bug-pending-exit-abort` の abort 機構は `spawn_blocking` の `std::thread::sleep` を中断できないため、drain 中の `pending_exit` 設定でも 30 秒待つ。本 issue はタイムアウトを短くするだけで根本解決はしない。0011 側で別機構を入れる想定。
- **0048 (過去履歴コメント削除) との関係**: 本 issue の修正対象である `transcode.rs:520-523` のコメントは提案コードで完全に置換される。0048 の作業対象は本 issue のコミットで消滅するため、0048 は本 issue 完了後に close する。
- **0018 との関係**: 本 issue で追加する `Error::Message` 1 箇所は 0018 での構造化バリアント置換対象 (0018 で `Error::DrainTimeout` 案あり)。0018 着手時に構造化バリアントへ置換される。本 issue は 0018 の前段にあたり、並行着手はできない。
- **tracing 初期化**: `tracing::warn!` の出力先は 0007 のテスト基盤整備後、main.rs の `tracing_subscriber` 初期化により利用可能。完了条件「ログが出力される」の検証は smoke test 実行時に行う。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **リソースリーク確認**: タイムアウト発生時、`encoder` は呼び出し元のスコープを抜けて `Drop` される。`shiguredo_video_toolbox::VtEncoder::Drop` が `VTCompressionSessionInvalidate` を呼ぶため安全。
- **CHANGES.md 追記**: `### 不具合修正` に「`drain_vt_encoder` の永久 busy-wait を 30 秒タイムアウトに変換」を 1 行で追記する。
