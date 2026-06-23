# 英語コメント (AGENTS.md「コメントは全て日本語にすること」違反)

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23

## 目的

AGENTS.md 規約に従って、ソースコード中の英語コメントを全て日本語に書き換える。

## 優先度根拠

AGENTS.md「コメントは全て日本語にすること」を満たしていない。9 箇所の英語コメントが存在。

## 現状

- `src/encode/sample_entry.rs:29, 30, 31` `// 4:2:0`, `// 8-bit`
- `src/encode/sample_entry.rs:46` `// VPS=32, SPS=33, PPS=34 (HEVC NAL unit types)`
- `src/encode/sample_entry.rs:70` `// Main profile`
- `src/encode/sample_entry.rs:76, 105` `// 4:2:0`
- `src/encode/transcode.rs:978` `// bps = total_bytes * 8 * timescale / total_duration_units`
- `src/encode/transcode.rs:1098` `// finalize`
- `prek.toml:1-2` `# Configuration file for \`prek\`, ...`, `# See https://prek.j178.dev for more information.`

## 設計方針

全て日本語に置き換える。技術略語 (`MP4`, `H.264`, `NAL`, `SPS`, `PPS`, `VPS`) は日本語コメント内に併記してもよい。

## 完了条件

上記箇所のコメントが全て日本語になる。`prek.toml` の英語コメント 2 行は prek 公式テンプレートの可能性が高いため、AGENTS.md 側で例外規定を追加するか、prek 設定ファイルを自前で再構築する。

## 解決方法

- `// 4:2:0` → `// クロマフォーマット 4:2:0` (または `// 4:2:0 (YUV420)`)
- `// 8-bit` → `// 8 ビット`
- `// VPS=32, SPS=33, PPS=34 (HEVC NAL unit types)` → `// VPS=32、SPS=33、PPS=34 (HEVC NAL ユニットタイプ)`
- `// Main profile` → `// Main プロファイル`
- `// bps = total_bytes * 8 * timescale / total_duration_units` → `// ビットレート (bps) = 合計バイト数 × 8 × タイムスケール / 合計期間単位`
- `// finalize` → `// muxer をファイナライズする`
- `prek.toml:1-2` → `# prek 設定ファイル (Rust 製の git hook フレームワーク)` / `# 詳細は https://prek.j178.dev を参照`

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0008 → 0016 (本 issue)** の順で develop に直接コミットする。0007 と 0009 の双方が closed になるまで本 issue は着手不可。
- **0008 との関係**: 0008 (`bug-ui-strings-japanese`) は UI 文字列 (リテラル) の日本語化で、本 issue はコメントの日本語化。直接 conflict しないが、AGENTS.md 規約違反修正の同タイミング着手で PR/review 見通しが悪化するため、**0008 → 0016 の順** でコミットする。
- **0007 で整備される基盤**: ドキュメント/コメントのみの変更なので新規テスト不要。0007 のテスト基盤は不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: 完了条件の clippy 要件をローカルで確認すること。
- **CHANGES.md 追記**: 完了条件の CHANGES.md 追記文言を参照。
