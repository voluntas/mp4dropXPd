# 英語コメント (AGENTS.md「コメントは全て日本語にすること」違反)

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-english-comments-japanese
- Polished:
- Reporter:

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
