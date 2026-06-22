# ドキュメント・コード間の UI 役割分担を README で明示

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/doc-clarify-ui-roles
- Polished:
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
