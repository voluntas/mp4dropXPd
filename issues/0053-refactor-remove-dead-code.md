# remove dead code (AppSettings fields + DropState status)

- Priority: Low
- Created: 2026-06-23
- Completed:
- Model: opencode-go/minimax-m3
- Branch: (CODEBASE.md によりブランチ不要 — develop に直接コミット)
- Polished: 2026-06-23
- Reporter:

## 目的

0024 (AppSettings::recipe / output_dir の dead fields 削除) と 0040 (DropState::status フィールドと idle_hint メソッドの削除) を統合し、1 回のコミットで両方のデッドコードを削除する。

## 優先度根拠

`#[expect(dead_code)]` で抑制されているフィールド (`recipe` / `output_dir`) および書き込まれるが参照されないフィールド (`status`) は YAGNI 違反のデッドコード。削除しても動作に影響はない。

## 現状

- `src/settings.rs:9-13`: `recipe` / `output_dir` フィールド。コードベース全体で読み取りは 0 件
- `src/app.rs:42`: `status: SharedString`。`Render for DropWindow` で表示されていない

## 設計方針

YAGNI に従い、使用されていないフィールド・メソッド・関連コードをすべて削除する。

## 完了条件

- `AppSettings::recipe` / `output_dir` フィールドと関連初期化コードが削除される
- `DropState::status` フィールドと `idle_hint` メソッドおよび関連呼び出しが削除される
- `cargo build` / `cargo clippy` が通過する

## 解決方法

### src/settings.rs

- `#[expect(dead_code)] pub recipe: EncodeRecipe,` を削除
- `#[expect(dead_code)] pub output_dir: PathBuf,` を削除
- `Default::default()` 内の `recipe: EncodeRecipe::default(),` を削除
- `Default::default()` 内の `output_dir: PathBuf::from("."),` を削除

### src/app.rs

- `pub status: SharedString,` (42 行目) を削除
- `DropState::new` 内の `status: "Drop MP4 here · 右クリックで設定".into(),` を削除
- `fn idle_hint(&mut self)` メソッド全体を削除
- `add_paths` 内の `self.status = "MP4 only".into();` (88 行目) を削除
- `add_paths` 内の `self.idle_hint();` (90 行目) を削除
- `poll_shared` 内の `self.state.idle_hint();` (115・137 行目) を削除
- `schedule_encode` 内の `self.state.status = ...` (160 行目) を削除
- `schedule_encode` 内の `view.state.status = summary.into();` (233 行目) を削除
