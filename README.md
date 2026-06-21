# mp4dropXPd

[oggdropXPd](https://rarewares.org/ogg-oggdropxpd.php) にインスパイアされたドラッグ＆ドロップ MP4 → MP4 再エンコーダ（macOS）。映像＋音声トラックのみ。

- UI: [GPUI](https://github.com/zed-industries/zed)（git submodule `extern/zed`）
  - GPUI は `extern/zed` に git submodule として取得し、`path` 依存で参照する
- 映像: `shiguredo_video_toolbox`（H.264 / H.265）、`shiguredo_svt_av1`（AV1）
- 音声: `shiguredo_audio_toolbox`（AAC）、`shiguredo_opus`（Opus）
- コンテナ: `shiguredo_mp4`
- 映像入力デコード: `shiguredo_video_toolbox`（H.264/H.265）、`shiguredo_dav1d`（AV1）
- 音声入力デコード: `shiguredo_audio_toolbox`（AAC / Opus）

## スクリーンショット

<img src="https://i.gyazo.com/ca0cac9ebd0b3104a615725e142b19ac.png" width="240" alt="mp4dropXPd screenshot">

## 機能

- メインウィンドウに `.mp4` をドロップすると**自動的にエンコード**が始まる
- **右クリック** で oggdropXPd 風のコンテキストメニュー
  - Options…
  - Audio (AAC / Opus)
  - Video (H.264 / H.265 / AV1)
  - Auto-encode on drop / Overwrite output
  - Clear / Quit
- **Options…** ウィンドウ（右クリックから）
  - Audio / Video codec 選択
  - 自動エンコード、出力上書き
  - ビットレート（Audio / Video）— デフォルトは入力の実ビットレートを引き継ぐ (Auto)

## ビルド

```sh
git submodule update --init --depth 1    # Zed (GPUI) の取得
xcodebuild -downloadComponent MetalToolchain   # 必要な場合のみ
cargo build
```

## 実行

```sh
cargo run
```

## 出力ファイル名

ドロップ時は入力ファイルと同じディレクトリに `<stem>.<video>_<audio>.mp4` で出力される。

例: `sample.mp4` → `sample.AV1_opus.mp4`
