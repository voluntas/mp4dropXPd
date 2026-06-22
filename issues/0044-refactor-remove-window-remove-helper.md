# window_remove ヘルパーと長文設計メモコメントの削除 (#0022 と統合)

- Priority: Low
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/refactor-remove-window-remove-helper
- Polished:
- Reporter:

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
