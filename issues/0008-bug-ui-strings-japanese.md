# UI 文字列がほぼ全て英語 (AGENTS.md「常に日本語を利用すること」違反)

- Priority: High
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-ui-strings-japanese
- Polished:
- Reporter:

## 目的

ユーザー向け UI 文字列 (メニュー・ステータス・エラーメッセージ等) を全て日本語に統一する。

## 優先度根拠

AGENTS.md「常に日本語を利用すること」「コメントは全て日本語にすること」は UI 文字列にも適用されるべき。現状は 35 箇所以上で英語ベースの UI 文字列が使われており、規約違反。

## 現状

`src/app.rs:56, 69, 88, 160, 216, 223, 226, 346, 351, 354, 357, 423, 429, 446, 455, 590, 593, 617, 687, 689, 728, 744, 782, 819, 879, 902, 925, 927, 950, 969, 996, 1059, 1086, 1167, 1181, 1184` 等に英語文字列:

- `"Drop MP4 Here"`
- `"Right-click for menu · Ctrl+Click"`
- `"Audio codec"` / `"Video codec"` / `"Behavior"`
- `"Audio bitrate"` / `"Video bitrate"`
- `"Auto"` / `"ON"` / `"OFF"`
- `"Saved"` / `"Close"` / `"Save"`
- `"Options…"` / `"Encode"` / `"Clear"` / `"Exit"`
- `"Done (ok)"` / `"Task: …"` / `"Ready"`
- `"MP4 only"` / `"Encoding…"`
- `"Changes apply on Save"`

## 設計方針

全ての UI 文字列を日本語に統一する。リソースファイルへの切り出しは将来の改善として残し、今回は文字列リテラルを直接日本語化する。

## 完了条件

`src/app.rs` 内の UI 文字列が全て日本語になる。英語文字列 (技術用語 "MP4", "H.264" 等を除く) がゼロになる。

## 解決方法

各文字列を以下のように置換する (例):

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
- `"Task: …"` → `"タスク: …"`
- `"Ready"` → `"準備完了"`
- `"MP4 only"` → `"MP4 のみ"`
- `"Encoding…"` → `"エンコード中…"`
- `"Changes apply on Save"` → `"保存ボタンで反映"`
