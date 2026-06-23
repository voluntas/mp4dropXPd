# encode_tracks の独立失敗処理導入 (部分成功の保持)

- Priority: High
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

`encode_tracks` で映像と音声を独立に評価し、片方失敗時にもう片方の結果を破棄しないようにする。本 issue のスコープは `encode_tracks` の API 変更 (`Result` から `EncodeTracksOutput` へ) に限定する。UI への部分成功情報伝播は後続の 0017 で対応する。

## 優先度根拠

現状は音声エンコードの 1 回の失敗で映像側も全て破棄され、最初から再エンコードが必要。ユーザー体験とリソース効率の双方で問題。`transcode.rs:189, 200` の `?` 直伝播が片方失敗で全体停止させている。

## 現状

`src/encode/transcode.rs:174-205` `encode_tracks`:

```rust
let video_output = if !video_samples.is_empty() {
    Some(encode_video(...)?)  // ? で抜けると audio_output は計算されない
} else { None };
let audio_output = if !audio_samples.is_empty() {
    Some(encode_audio(...)?)
} else { None };
```

`?` で関数全体を抜けるため、片方が失敗するともう片方の結果も破棄される。両方が失敗したときのエラー情報も 1 つの `Error` に潰される。

## 設計方針

映像と音声を独立に `match` で評価し、それぞれの成否を呼び出し側に返す型を導入する。`encode_tracks` の戻り値型を `Result<(Option<VideoOutput>, Option<AudioOutput>)>` から `EncodeTracksOutput` に変更する。

`EncodeTracksOutput` と `TrackError` は `#[cfg(target_os = "macos")]` ガード下に定義する。`VideoOutput` / `AudioOutput` が `#[cfg(target_os = "macos")]` (`transcode.rs:72-88`) 内でのみ定義されているためである。non-macOS 側の `encode_tracks` は `Ok(None)` を返すのみのため `EncodeTracksOutput` の non-macOS 版を cfg 分岐で定義する。

各トラックの失敗時は `TrackError::Video(String)` / `TrackError::Audio(String)` に `Error::to_string()` でメッセージを格納し、`tracing::warn!` でログを残す。`TrackError` は `Display` を実装する。本 issue では文字列ベースの暫定型に留め、0018 での構造化を前提とする。0018:50 の `Error::TrackEncode { track, message }` と機能重複するが、本 issue 導入時点では独立型として運用し、0018 着手時に `Error::TrackEncode` に統合する。

## 完了条件

- `encode_tracks` が `EncodeTracksOutput` を返し、映像と音声の成否を独立に保持する
- 映像成功・音声失敗のケースで `encode_tracks` がクラッシュせず `EncodeTracksOutput { video: Ok(Some(...)), audio: Err(TrackError(...)) }` を返す
- 映像失敗・音声成功のケースで同様に `video: Err(...), audio: Ok(Some(...))` を返す
- 両方失敗時に `video: Err(...), audio: Err(...)` を返し、両方のエラー情報が保持される
- `TrackError` が `Display` を実装し、`tracing::warn!` でエラー内容がログ出力される
- 進捗 (`JobProgress`) は映像フレーム数基準で計算し、エンコード失敗時は既に処理済みのフレーム分まで進捗が進んだ状態で復帰する
- `cargo clippy --all-targets -- -D warnings` を通過する
- [FIX] encode_tracks を部分成功対応に変更 (映像/音声を独立評価、片方失敗時に残りを保持)

## 解決方法

`src/encode/transcode.rs` に以下を追加する。`TrackEncodeResult` / `TrackError` / `EncodeTracksOutput` は `encode_tracks` の直前に定義する。`#[cfg(target_os = "macos")]` ガード下と non-macOS 用の cfg 分岐の両方を用意する。

```rust
#[cfg(target_os = "macos")]
type TrackEncodeResult<T> = std::result::Result<Option<T>, TrackError>;

#[cfg(target_os = "macos")]
#[derive(Debug)]
enum TrackError {
    Video(String),
    Audio(String),
}

#[cfg(target_os = "macos")]
impl std::fmt::Display for TrackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Video(s) => write!(f, "video: {s}"),
            Self::Audio(s) => write!(f, "audio: {s}"),
        }
    }
}

#[cfg(target_os = "macos")]
struct EncodeTracksOutput {
    video: TrackEncodeResult<VideoOutput>,
    audio: TrackEncodeResult<AudioOutput>,
}

#[cfg(not(target_os = "macos"))]
struct EncodeTracksOutput {
    video: Result<Option<()>, ()>,
    audio: Result<Option<()>, ()>,
}
```

修正後の `encode_tracks` (macOS 版、戻り値型を変更):

```rust
#[cfg(target_os = "macos")]
fn encode_tracks(
    video_samples: &[RawSample],
    audio_samples: &[RawSample],
    video_timescale: Option<NonZeroU32>,
    audio_timescale: Option<NonZeroU32>,
    recipe: EncodeRecipe,
    progress: &JobProgress,
) -> EncodeTracksOutput {
    let video = if !video_samples.is_empty() {
        match encode_video(
            video_samples,
            video_timescale,
            recipe.video,
            recipe.video_bitrate_kbps,
            progress,
        ) {
            Ok(output) => Ok(Some(output)),
            Err(e) => {
                tracing::warn!("video encode failed: {e}");
                Err(TrackError::Video(e.to_string()))
            }
        }
    } else {
        Ok(None)
    };

    let audio = if !audio_samples.is_empty() {
        match encode_audio(
            audio_samples,
            audio_timescale,
            recipe.audio,
            recipe.audio_bitrate_kbps,
        ) {
            Ok(output) => Ok(Some(output)),
            Err(e) => {
                tracing::warn!("audio encode failed: {e}");
                Err(TrackError::Audio(e.to_string()))
            }
        }
    } else {
        Ok(None)
    };

    EncodeTracksOutput { video, audio }
}
```

`transcode` 関数 (line 90-171) の修正。`encode_tracks(...)?` を `encode_tracks(...)` に変更し、戻り値型の `?` 伝播を外す:

```rust
// transcode.rs:159 付近。現状の `let (video_output, audio_output) = encode_tracks(...)?;`
// を以下に置き換える
let encode_output = encode_tracks(
    &video_samples, &audio_samples,
    video_timescale, audio_timescale,
    recipe, progress,
);
// 成功したトラックだけを取り出す。所有権を move する。
let video_for_mux = match encode_output.video {
    Ok(Some(v)) => Some(v),
    _ => None,
};
let audio_for_mux = match encode_output.audio {
    Ok(Some(a)) => Some(a),
    _ => None,
};
// 両方とも出力不能なら早期 return (空入力または両方失敗)
if video_for_mux.is_none() && audio_for_mux.is_none() {
    return Ok(());
}
write_mp4(output, video_for_mux, audio_for_mux, video_timescale)?;
```

`transcode` の戻り値型は `Result<()>` のままとする。`EncodeTracksOutput` は `transcode` 内部で消費され、現時点では呼び出し元に伝播しない。部分成功情報の UI 伝播は 0017 で対応する。

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0005 (本 issue)** の順で develop に直接コミットする。
- **0007 で整備される基盤**: `tests/test_encode.rs` へのテスト追加は 0007 完了後に行う。
- **エンコード失敗のテスト方法**: 不正なビットレート値 (0 や極大値) を指定する、または不正な `RawSample` データを用意することでエンコーダのエラーを誘発する。具体的な方法はテスト実装時に検討する。
- **non-macOS 版 `encode_tracks`**: `Ok(None)` を返すダミー実装に変更する。`transcode` 本体側の `encode_tracks(...)?` の `?` を外すため、non-macOS 側 `transcode` も同様に修正する。
- **0017 との関係**: 0017 で `transcode` の戻り値型を `Result<EncodeTracksOutput>` に拡張し、UI への部分成功情報伝播を行う。本 issue はその前段として `encode_tracks` の API を整備する。
- **0018 との関係**: 本 issue で導入する `TrackError` は 0018 着手時に `Error::TrackEncode { track, message }` に統合する。0018:50 に統合計画を追記済み。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
