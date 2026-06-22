# window_remove ヘルパーが誤った前提 + メニュー多重起動防止なし

- Priority: Medium
- Created: 2026-06-22
- Completed:
- Model: opencode-go/minimax-m3
- Branch: feature/fix-window-remove-and-menu-dedupe
- Polished:
- Reporter:

## 目的

(1) `window_remove` ヘルパーを削除し、listener クロージャの `_w: &mut Window` から直接 `remove_window()` を呼ぶ。(2) `open_context_menu` の入口に多重起動防止チェックを追加する。

## 優先度根拠

(1) ヘルパーのコメント「`cx` から window にアクセスする方法がない」は誤りで、`OptionsWindow::on_close` (line 1164-1166) では `_w.remove_window()` を直接呼んでいる。(2) 連続右クリックで `MenuWindow` が複数開く。

## 現状

`src/app.rs:750-761`:

```rust
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

`src/app.rs:273-321` `open_context_menu` の入口は `if self.state.encoding { return; }` のみで `self.menu_open` チェックなし。

## 設計方針

(1) ヘルパーと `pending_close_menu` フラグ、`MenuWindow::render` 冒頭の消費ロジック (line 557-565) をすべて削除。listener クロージャの `_w.remove_window()` を直接呼ぶ。(2) `open_context_menu` の入口に `if self.menu_open { return; }` を追加。

## 完了条件

`window_remove` ヘルパーと関連フラグ/ロジックが削除される。メニュー項目クリックで `MenuWindow` が直接閉じる。連続右クリックで `MenuWindow` が 1 個だけ表示される。

## 解決方法

1. `src/app.rs:750-761` の `window_remove` 関数を削除
2. `src/app.rs:588, 708, 726, 742` の `window_remove(cx)` 呼び出しを `w.remove_window()` (listener クロージャの `_w` から) に置換
3. `src/app.rs:557-565` の `pending_close_menu` 消費ロジックを削除
4. `src/app.rs:20-29, 145-148` の `pending_close_menu` フィールドと消費ロジックを削除
5. `src/app.rs:279` の `if self.state.encoding { return; }` の直後に `if self.menu_open { return; }` を追加
