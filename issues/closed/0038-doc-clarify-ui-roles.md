# ドキュメント・コード間の UI 役割分担を README で明示

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

`README.md` でコンテキストメニューと Options ウィンドウの UI 役割分担を明確化する。

## 優先度根拠

`README.md:22-23` は「**Auto-encode on drop**」「**Overwrite output**」が「**Options…** ウィンドウ」の設定項目として説明されているが、UI 上はコンテキストメニュー (右クリック) にも置かれている。ユーザー混乱の元。

## 現状

`README.md:19-25` で「メインウィンドウに `.mp4` をドロップすると自動的にエンコード」「右クリックで oggdropXPd 風のコンテキストメニュー (Options / Audio / Video / Auto-encode on drop / Overwrite output / Clear / Quit)」と説明。

`Options…` のみが別ウィンドウで、Audio/Video 切替はコンテキストメニュー内。README はこの区別を明確に書いていない。

## 設計方針

README に以下の役割分担を明記する。

- コンテキストメニュー (右クリック): クイックトグル (よく切り替える設定)
- Options ウィンドウ: 詳細設定 (ビットレート等)

## 完了条件

README に「コンテキストメニュー」と「Options ウィンドウ」の役割分担が明記される。

## 解決方法

`README.md:22-23` の機能リストに以下を追加する。

> コンテキストメニューはクイックトグル、Options ウィンドウは詳細設定 (ビットレート等) の役割です。Auto-encode on drop / Overwrite output は両方から変更可能で、即座に反映されます。

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0038 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0008 との関係**: 0008 (UI 文字列日本語化) で `src/app.rs:19-25` 相当の英語 UI 文字列が日本語化される。README の英語 UI 文字列 (`Auto-encode on drop` / `Overwrite output` / `Options…` 等) も 0038 で日本語化する。**0008 と並行可**。
- **0010/0041/0049 との関係**: 0010 (`overwrite` 機能実装) と 0041/0049 (`overwrite` UI 削除) の判断が出てから 0038 を確定する (Overwrite output の説明が要不要が変わる)。
- **0007 で整備される基盤**: ドキュメントのみなのでテスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: ドキュメント変更のみなので影響なし。
- **CHANGES.md 追記**: `### misc` サブセクション (0009 で確立) に `[DOC] README にコンテキストメニューと Options ウィンドウの役割分担を明記` を 1 行で追記する。
