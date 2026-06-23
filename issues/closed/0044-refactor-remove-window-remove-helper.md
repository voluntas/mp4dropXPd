# window_remove ヘルパーと長文設計メモコメントの削除 (#0022 と統合)

- Priority: Low
- Created: 2026-06-22
- Completed: 2026-06-23
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:
- Status: **0022 に統合してクローズ推奨** (本 issue の作業内容は 0022 に完全包含)

## 目的

`src/app.rs:750-761` の `window_remove` ヘルパーと 6 行の設計メモコメントを削除する。

## 優先度根拠

ヘルパーは `MenuWindow::request` (line 543-546) と等価 (両方とも `pending_close_menu = true` をセットする)。6 行の設計メモは「`cx` から window にアクセスする方法がない」と誤った情報を記載しているが、実際には `_w: &mut Window` でアクセス可能。

## 現状

`src/app.rs:750-761`:

```rust
/// remove_window を安全に呼ぶヘルパー
/// cx は Context<MenuWindow>
fn window_remove(cx: &mut Context<MenuWindow>) {
    // on_click の listener は (&mut MenuWindow, &ClickEvent, &mut Window, &mut Context<MenuWindow>)
    // だが cx.listener のクロージャは (&mut MenuWindow, &E, &mut Window, &mut Context<MenuWindow>)
    // なので、実際には _w が Window
    // しかし上記の on_click では _w を無視している
    // 代わりに cx から window にアクセスする方法がないので、
    // グローバルフラグで閉じる
    let mut s = shared(cx);
    s.pending_close_menu = true;
}
```

呼び出し元: `src/app.rs:588, 708, 726, 742`

## 設計方針

#0022 (window_remove ヘルパーが誤った前提) と統合して対応。ヘルパー削除、listener クロージャの `_w.remove_window()` を直接呼ぶ形に書き換える。

## 完了条件

`window_remove` ヘルパーと 6 行の設計メモコメントが削除される。メニュー項目クリックで `MenuWindow` が直接閉じる。

## 解決方法

#0022 を参照して対応。具体的には:

1. `src/app.rs:750-761` の `window_remove` 関数を削除
2. `src/app.rs:588, 708, 726, 742` の `window_remove(cx)` 呼び出しを `w.remove_window()` (listener クロージャの `_w` から) に置換

実装時の確認手順:

- **依存順序**: **0007 (テスト基盤整備) → 0009 (CHANGES.md 新規作成) → 0022 (本 issue は 0022 と統合)** の順で develop に直接コミットする。0022 (window_remove 削除) と本 issue は重複しているため、**0022 に統合** して 0022 内で対応する。0044 は close。
- **0001-0008 との並行**: 0022 と統合後、`src/app.rs:588, 708, 726, 742` の修正は 0001-0008 とは作業領域が重ならない。並行可。
- **0007 で整備される基盤**: 新規テスト不要。
- **0018 との関係**: 本 issue は `Error::Message` を追加しない。0018 への影響なし。
- **clippy 通過**: `cargo clippy --workspace --all-targets -- -D warnings` がローカルで 0 warning で完了することを確認。
- **CHANGES.md 追記**: 0022 側で `### 不具合修正` サブセクション (0009 で確立) に `[FIX] window_remove ヘルパーと pending_close_menu フラグを削除、メニュー多重起動防止チェックを追加` を 1 行で追記する (本 issue は 0022 に統合)。
