# 映像/音声エンコードの独立失敗処理がなく部分成功が利用できない

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-encode-tracks-partial-success
- Polished:
- Reporter:

## 目的

映像エンコードが成功して音声エンコードが失敗した場合に、映像側の結果を破棄せず、UI で「部分成功」状態を提示できるようにする。

## 優先度根拠

現状は音声エンコードの 1 回の失敗で映像側も全て破棄され、最初から再エンコードが必要。ユーザー体験とリソース効率の双方で問題。

## 現状

`src/encode/transcode.rs:174-205` `encode_tracks`:

```rust
let video_output = if !video_samples.is_empty() {
    Some(encode_video(...)?)  // ←ここで ? で抜けると audio_output は計算されない
} else { None };
let audio_output = if !audio_samples.is_empty() {
    Some(encode_audio(...)?)
} else { None };
```

`?` で関数全体を抜けるため、片方が失敗するともう片方の結果も破棄される。

## 設計方針

映像と音声を独立したトランザクションとして扱い、両方の結果を呼び出し側に返す。失敗時は `Result<(Option<VideoOutput>, Option<AudioOutput>), Error>` で部分成功を許容する。

## 完了条件

映像成功・音声失敗のケースで、UI に「映像は成功、音声は失敗」と表示できる (あるいは音声のみの再エンコードを案内できる)。現状のような「全部やり直し」にならない。

## 解決方法

`src/encode/transcode.rs:174-205` を以下のように修正する。

- 戻り値型を `Result<(Option<VideoOutput>, Option<AudioOutput>), Error>` に変更
- `let video_result = if !video_samples.is_empty() { Some(encode_video(...).await?) } else { None };`
- `let audio_result = if !audio_samples.is_empty() { Some(encode_audio(...).await?) } else { None };`
- 失敗時はログを残しつつ部分結果を返す
- `mux` 段階で部分結果に応じて適切に処理
