# UI 文字列がほぼ全て英語 (AGENTS.md「常に日本語を利用すること」違反)

- Priority: High
- Created: 2026-06-22
- Completed: 2026-06-23
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

ユーザー向け UI 文字列 (メニュー・ステータス・エラーメッセージ等) を全て日本語に統一する。`src/app.rs` および `src/codec.rs` の `AudioCodec::label()` / `VideoCodec::label()` / `EncodeRecipe::summary()` が返す文字列を、AGENTS.md「常に日本語を利用すること」「コメントは全て日本語にすること」に従い日本語化する (技術用語例外を除く)。

## 優先度根拠

AGENTS.md「常に日本語を利用すること」は UI 文字列にも適用されるべき。現状は 50 箇所以上で英語 UI 文字列が使われており、規約違反。アプリ利用者 (日本語話者) への直接的な影響。

## 現状

`src/app.rs` 内に 50 箇所以上の英語 UI 文字列。主な箇所 (主要な行のみ抜粋、完全な一覧は後述):

- `src/app.rs:56, 69` `status: "Drop MP4 here · 右クリックで設定".into()` (一部日本語化済)
- `src/app.rs:88` 進捗フォーマット
- `src/app.rs:160` 完了表示
- `src/app.rs:216` Ready 状態
- `src/app.rs:226` `format!("Task: {e:?}")` ← **0017 に移管 (panic 情報漏洩対応)**
- `src/app.rs:249, 877` ウィンドウタイトル `"mp4dropXPd — Options"`
- `src/app.rs:346, 351, 354, 357` メニュー項目
- `src/app.rs:423, 429` メイン画面の太字タイトル
- `src/app.rs:446, 455` 設定画面
- `src/app.rs:590, 593, 617` メニュー項目
- `src/app.rs:657, 659, 678, 680` トグル項目
- `src/app.rs:687, 689, 728, 744, 782` UI テキスト
- `src/app.rs:819, 879, 902, 925, 927, 950, 969, 996` 設定画面
- `src/app.rs:944, 946, 957, 965` `"ON"` / `"OFF"` トグル
- `src/app.rs:1059, 1086, 1167, 1181, 1184` 残り
- `src/app.rs:1205` ウィンドウタイトル `"mp4dropXPd"`

`src/codec.rs:11-15` `AudioCodec::label()` → `"AAC"`, `"Opus"`、`src/codec.rs:30-34` `VideoCodec::label()` → `"H.264"`, `"H.265"`, `"AV1"`、`src/codec.rs:48-50` `EncodeRecipe::summary()` → `"AAC + AV1"` 等の英語文字列。UI 側 (`src/app.rs:594-615, 618-639, 885-900, 907-923`) で多用される。

## 設計方針

UI 文字列を全て日本語化する。リソースファイル (`i18n` 等) への切り出しは将来の改善として残し (別 issue `add-i18n-resource-file` で起票予定)、今回は文字列リテラルを直接日本語化する。技術用語例外を以下のように明文化する:

**技術用語例外 (翻訳しない、英語のまま)**:

- codec 名: `"MP4"`, `"H.264"`, `"H.265"`, `"AV1"`, `"AAC"`, `"Opus"`, `"VP9"` (将来対応)、`"GPUI"`
- 単位: `"kbps"`, `"Hz"`, `"fps"`
- 拡張子: `".mp4"`, `".tmp"`
- 固有名詞: `"mp4dropXPd"` (アプリ名)
- 記号: `+` (`summary()` の `" + "` 区切り)、`"` `"` 引用符

**翻訳対象 (主要 50+ 項目の例示、完全な一覧は「解決方法」参照)**:

- `"Drop MP4 Here"` → `"MP4 をドロップ"`
- `"Right-click for menu · Ctrl+Click"` → `"右クリックでメニュー · Ctrl+クリック"`
- `"Audio codec"` → `"音声コーデック"`
- `"Video codec"` → `"映像コーデック"`
- `"Behavior"` → `"動作"`
- `"Audio bitrate"` → `"音声ビットレート"`
- `"Video bitrate"` → `"映像ビットレート"`
- `"Auto"` → `"自動"`
- `"ON"` / `"OFF"` → `"オン"` / `"オフ"`
- `"Saved"` → `"保存しました"`
- `"Close"` → `"閉じる"`
- `"Save"` → `"保存"`
- `"Options…"` → `"オプション…"`
- `"Encode"` → `"エンコード"`
- `"Clear"` → `"クリア"`
- `"Exit"` → `"終了"`
- `"Done (ok)"` → `"完了 ({ok})"`
- `"Task: …"` → **0017 移管 (本 issue では扱わない)**
- `"Ready"` → `"準備完了"`
- `"MP4 only"` → `"MP4 のみ"`
- `"Encoding…"` → `"エンコード中…"`
- `"Changes apply on Save"` → `"保存ボタンで反映"`
- `"mp4dropXPd — Options"` → `"mp4dropXPd — オプション"`
- `"● Auto-encode on drop"` → `"● ドロップ時自動エンコード"`
- `"● Overwrite output"` → 0010/0041/0049 の判断待ち (本 issue では扱わない)
- `"{kbps} kbps"` → `"{kbps} kbps"` (技術用語例外)

## 完了条件

- `src/app.rs` および `src/codec.rs` 内の UI 文字列が、技術用語例外 (上記) を除いて全て日本語になる
- 翻訳済みなのに再度リストされている行 (`src/app.rs:56, 69` 等) は混在文字列として統一的に扱う
- `src/app.rs:226` の `Task: …` は本 issue のスコープ外 (0017 で完全置換)
- 0010 / 0041 / 0049 の `overwrite` 関連 UI は本 issue のスコープ外
- 機械的検査: `rg '"[A-Z][a-z]+ [a-z]' src/app.rs src/codec.rs` で英語 UI 文字列の残存が技術用語例外のみになることを確認するワンライナーを `prek.toml` に追加 (または `CONTRIBUTING.md` に記載)
- `tests/test_codec.rs` で `AudioCodec::label()` / `VideoCodec::label()` の戻り値が技術用語例外 (`"AAC"`, `"Opus"`, `"H.264"`, `"H.265"`, `"AV1"`) と一致することを assert する単体テストが追加されている
- `cargo clippy --all-targets -- -D warnings` (`prek.toml:28`) を通過する
- [FIX] UI 文字列を全て日本語化 (技術用語例外を除く)

## 解決方法

### 段階導入プラン

35+ 箇所を一気に置換するリスクを避けるため、3 段階で導入する。各段で `git diff` のレビュー単位を 200 行以内に収める。

**第 1 段 (20 箇所)**: ウィンドウタイトル + メニュー項目 + ステータス表示

- `src/app.rs:249, 877, 1205` ウィンドウタイトル
- `src/app.rs:346, 351, 354, 357, 590, 593, 617, 657, 659, 678, 680, 687, 689, 728, 744, 782` メニュー項目

**第 2 段 (15 箇所)**: OptionsWindow のラベル

- `src/app.rs:346, 351, 354, 357, 423, 429, 446, 455, 879, 902, 925, 927, 950, 969, 996` 設定画面ラベル

**第 3 段 (15 箇所)**: 細かいラベル + ステータステキスト

- `src/app.rs:88, 160, 216, 819, 1059, 1086, 1167, 1181, 1184` 残り
- `src/app.rs:944, 946, 957, 965` `"ON"` / `"OFF"` トグル
- `src/app.rs:335` の `_status` デッドコードは 0039 (`refactor-remove-dead-status-clone`) で削除

`src/codec.rs` の `label()` / `summary()` は技術用語例外に含めるため変更しない。`label()` 戻り値の固定値検証は `tests/test_codec.rs` に追加。

### 機械的検査の追加

`prek.toml` に以下のローカル hook を追加 (もしくは `CONTRIBUTING.md` に手順記載):

```bash
# 英語 UI 文字列の残存検出 (技術用語例外を除く)
rg '"[A-Z][a-z]+( [a-z]+)+"' src/app.rs src/codec.rs
```

期待される出力は技術用語例外 (例: `"Drop MP4 here"` 等) のみで、本 issue 完了後は 0 件。

実装時の確認手順:

- **依存順序**: 完了条件 4, 5, 6 を満たすには前段 issue が必要。**0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0008 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0016 との関係**: 0016 (`bug-english-comments-japanese`) は `src/encode/sample_entry.rs` と `src/encode/transcode.rs` のコメントを編集する。直接 conflict しないが、AGENTS.md 規約違反修正の同タイミング着手で PR/review 見通しが悪化するため、**0008 → 0016 の順** でコミットする。
- **0017 との関係**: 0017 (`bug-join-error-ui-disclosure`) は `src/app.rs:226` の `format!("Task: {e:?}")` を `e.is_panic()` 判定で完全置換する。**0008 では line 226 を扱わない** (0017 に完全移管)。`Task: …` 翻訳を 0008 に含めないこと。
- **0010 / 0041 / 0049 との関係**: `Overwrite output` 関連 UI (`src/app.rs:678, 680`) は 0010 (`bug-overwrite-flag-wiring`) の機能実装判断待ち。0010 で残す判断なら 0008 で翻訳、0041/0049 で削除するなら 0008 では翻訳しない。**0008 → 0010 → 0041/0049 の順** で判断する。0008 着手時点で 0010 機能が未確定の場合、`Overwrite output` の翻訳は保留する。
- **0032 / 0046 との関係**: `tracing::warn!` の出力先確保は 0032 / 0046 の判断に従う。本 issue では UI 文字列の日本語化のみで、ログメッセージは英語のままとする (AGENTS.md「ログメッセージは全て英語にすること」)。UI とログの境界を混同しないこと。
- **0038 との関係**: `README.md:19-25` の機能リストも英語のまま。0038 (`doc-clarify-ui-roles`) で README 更新時に日本語化する想定。**0008 では `src/app.rs` と `src/codec.rs` のみ扱う**。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない (`src/app.rs:226` は 0017 移管のため)。0018 への影響なし。
- **0007 で整備される基盤**: `tests/test_codec.rs` への単体テスト追加は、0007 が `tests/` の Cargo 設定を済ませてから行う。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
