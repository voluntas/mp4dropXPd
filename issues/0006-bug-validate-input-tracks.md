# 入力 MP4 のトラック検証不足 (映像も音声もない MP4 で破損ファイル生成)

- Priority: High
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

映像トラックも音声トラックも含まない MP4 (例: 字幕のみ、データ/メタデータのみのトラック) を入力したときに、破損ファイルを生成せず `Result` で早期にエラーを返す。`shiguredo_mp4::TrackKind` には `Video` / `Audio` の 2 バリアントしか定義されておらず、それ以外のトラック種別 (字幕、データ、メタデータ) を含む MP4 も本 issue で `Err` として扱う。字幕/データトラックの mux 対応は別 issue で扱う想定。

## 優先度根拠

入力検証の欠落。破損ファイルの生成は UI/ユーザー双方に無用な混乱を与える。`shiguredo-rust` 規約「性能より堅牢性を優先」「`Result` で表現する」に従い、無効入力は早期 `Err` で停止する。

## 現状

`src/encode/transcode.rs:103-106`:

```rust
let tracks: Vec<TrackInfo> = demuxer.tracks()?.to_vec();

let video_idx = tracks.iter().position(|t| t.kind == TrackKind::Video);
let audio_idx = tracks.iter().position(|t| t.kind == TrackKind::Audio);
```

`video_idx` と `audio_idx` の両方が `None` のケースは検出されず、`write_mp4` まで到達して `None, None` を渡すことで破損 MP4 (`initial_bytes` のみ + サンプルなし) が生成される。

## 設計方針

`video_idx.is_none() && audio_idx.is_none()` のとき `Err` を返す。`position` で取得した index を後段の検証にも再利用する (line 105-106 で既に計算済みなので 2 度走査しない)。


## 完了条件

- 映像/音声トラックのどちらも持たない MP4 (例: 字幕のみ、データ/メタデータのみ、空 MP4) を入力すると、`Err(Error::Message("no video/audio track in input"))` が返り、破損ファイルが生成されない
- 映像/音声トラックのうち少なくとも一方が存在する MP4 では、引き続き正常処理される
- 0005 (`bug-encode-tracks-partial-success`) との整合性: 0005 case 1 (両方が `Ok(None)`) は「トラック種別は存在するがサンプルが 0 件」のケースを意味し、**「トラック自体が存在しない」ケース (本 issue の Err 対象) とは別**。0005 着手時に case 1 の説明に「トラックは存在するがサンプルなし」の注釈を追記する
- `tests/test_encode.rs` に字幕/データのみの MP4 を入力とするテストが追加されている
- `cargo clippy --all-targets -- -D warnings` (`prek.toml:28`) を通過する
- [FIX] 映像/音声トラックを含まない MP4 入力時の破損ファイル生成を防止 (早期 `Err` return)
- 失敗時に `tracing::warn!` で英語ログが出力される (AGENTS.md「ログメッセージは全て英語にすること」準拠)

## 解決方法

`src/encode/transcode.rs:103-106` 直後 (line 107 直前) に以下を追加する。line 105-106 で計算済みの `video_idx` / `audio_idx` を再利用する。

```rust
// 映像/音声トラックが一つもなければ出力対象がないため早期 return する
// (write_mp4 に None, None を渡すと破損 MP4 が生成されるため)
if video_idx.is_none() && audio_idx.is_none() {
    tracing::warn!("no video/audio track in input");
    return Err(Error::Message("no video/audio track in input".into()));
}
```

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0006 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0005 との関係**: 本 issue が「トラック自体が存在しない」段階で `Err` を返すため、0005 case 1 が「トラックなし」になることはない。0005 側の case 1 は「トラックは存在するがサンプルなし」のケースを意味する (0005 側の説明に注釈済み)。
- **0007 で整備される基盤**: `tests/test_encode.rs` へのテスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **テストフィクスチャ**: 字幕のみの MP4 フィクスチャは 0007 で整備される fixtures ディレクトリに配置する。`ffmpeg` で生成可能。
- **0018 との関係**: 本 issue で追加する `Error::Message` 1 箇所は 0018 の refactor 対象に含まれる。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
