use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use gpui::{
    App, Bounds, Context, ExternalPaths, FocusHandle, Focusable,
    MouseButton, MouseDownEvent, Styled, Task,
    Window, WindowBackgroundAppearance, WindowBounds,
    WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_tokio::Tokio;

use crate::codec::EncodeRecipe;
use crate::encode::{self, EncodeJob, JobProgress, default_output_path};
use crate::input::filter_mp4_paths;
use crate::settings::AppSettings;

use super::state::{shared, AppCommand};
use super::options_window::OptionsWindow;
use super::menu_window::MenuWindow;
use super::ui_helpers::bitrate_label;
#[derive(Clone, Debug)]
pub struct DropState {
    recipe: EncodeRecipe,
    dropped_paths: Vec<PathBuf>,
    encoding: bool,
    settings: AppSettings,
    /// エンコード進捗 (0.0 - 1.0)
    progress: f64,
    /// エンコード開始時刻 (ETA 計算用)
    encode_started: Option<Instant>,
}

impl DropState {
    fn new() -> Self {
        Self {
            recipe: EncodeRecipe::default(),
            dropped_paths: Vec::new(),
            encoding: false,
            settings: AppSettings::default(),
            progress: 0.0,
            encode_started: None,
        }
    }

    fn add_paths(&mut self, paths: &[PathBuf]) {
        let mp4 = filter_mp4_paths(paths);
        for p in mp4 {
            if !self.dropped_paths.contains(&p) {
                self.dropped_paths.push(p);
            }
        }
    }
}

pub struct DropWindow {
    state: DropState,
    focus: FocusHandle,
    menu_open: bool,
    encode_task: Option<Task<()>>,
}

impl DropWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        Self {
            state: DropState::new(),
            focus,
            menu_open: false,
            encode_task: None,
        }
    }

    fn poll_shared(&mut self, cx: &mut Context<Self>) {
        let mut s = shared(cx);
        while let Some(cmd) = s.commands.pop_front() {
            match cmd {
                AppCommand::UpdateRecipe(r) => {
                    self.state.recipe = r;
                }
                AppCommand::UpdateSettings(s2) => {
                    self.state.settings = s2;
                }
                AppCommand::OpenOptions => {
                    drop(s);
                    self.open_options(cx);
                    cx.notify();
                    return;
                }
                AppCommand::TriggerEncode => {
                    drop(s);
                    self.schedule_encode(cx);
                    cx.notify();
                    return;
                }
                AppCommand::Clear => {
                    self.state.dropped_paths.clear();
                }
                AppCommand::Exit => {
                    drop(s);
                    if let Some(task) = self.encode_task.take() {
                        task.detach();
                    }
                    cx.quit();
                    return;
                }
            }
        }
        drop(s);
    }

    fn schedule_encode(&mut self, cx: &mut Context<Self>) {
        if self.state.encoding || self.state.dropped_paths.is_empty() {
            return;
        }

        self.state.encoding = true;
        self.state.progress = 0.0;
        self.state.encode_started = Some(Instant::now());

        let mut recipe = self.state.recipe;
        recipe.video_bitrate_kbps = self.state.settings.video_bitrate_kbps;
        recipe.audio_bitrate_kbps = self.state.settings.audio_bitrate_kbps;
        let auto_clear = self.state.settings.auto_encode_on_drop;
        let jobs: Vec<EncodeJob> = self
            .state
            .dropped_paths
            .iter()
            .map(|input| EncodeJob::new(input.clone(), default_output_path(input, recipe), recipe, self.state.settings.overwrite))
            .collect();

        // 各ジョブの進捗を追跡する JobProgress を生成
        let job_count = jobs.len();
        let progresses: Vec<Arc<JobProgress>> = (0..job_count)
            .map(|_| Arc::new(JobProgress::new()))
            .collect();
        let progresses_for_poll = progresses.clone();

        self.encode_task = Some(cx.spawn(async move |this, cx| {
            // エンコード完了フラグ (Tokio スレッド → GPUI スレッド間で共有)
            let done = Arc::new(AtomicBool::new(false));
            let done_for_encode = done.clone();

            // エンコードタスクを Tokio ランタイムで起動
            let encode_task = Tokio::spawn(cx, {
                let done = done.clone();
                async move {
                    let result = encode::run_jobs_async(jobs, progresses).await;
                    done.store(true, Ordering::SeqCst);
                    result
                }
            });

            // 各ジョブの進捗通知ハンドルを収集する
            let notifies: Vec<Arc<tokio::sync::Notify>> = progresses_for_poll
                .iter()
                .map(|p| p.notify_handle())
                .collect();

            // 進捗をイベント駆動で待機しながらエンコード完了を待つ
            loop {
                // いずれかのジョブで進捗があれば即座に UI 更新する
                if let Some(n) = notifies.first() {
                    n.notified().await;
                }
                let total: f64 =
                    progresses_for_poll.iter().map(|p| p.ratio()).sum::<f64>() / job_count as f64;
                let _ = this.update(cx, |view, cx| {
                    view.state.progress = total;
                    cx.notify();
                });
                if done_for_encode.load(Ordering::SeqCst) {
                    break;
                }
            }

            let _summary = match encode_task.await {
                Ok(rows) => {
                    let ok = rows.iter().filter(|(_, r)| r.is_ok()).count();
                    let err = rows.len().saturating_sub(ok);
                    if err == 0 {
                        format!("完了 ({ok})")
                    } else {
                        let first = rows
                            .iter()
                            .find_map(|(_, r)| r.as_ref().err())
                            .map(|e| e.to_string())
                            .unwrap_or_default();
                        format!("{ok} 成功, {err} 失敗: {first}")
                    }
                }
                Err(e) => {
                    if e.is_panic() {
                        tracing::error!(error = %e, "encode task panicked");
                        "エンコードタスクがパニックしました".into()
                    } else {
                        tracing::error!(error = %e, "encode task failed");
                        "エンコードタスクが失敗しました".into()
                    }
                }
            };

            let _ = this.update(cx, |view, cx| {
                view.state.encoding = false;
                view.state.progress = 1.0;
                view.state.encode_started = None;
                view.encode_task = None;
                if auto_clear {
                    view.state.dropped_paths.clear();
                }
                cx.notify();
            });
        }));
    }

    fn open_options(&mut self, cx: &mut App) {
        let recipe = self.state.recipe;
        let settings = self.state.settings.clone();
        let _ = cx.open_window(
            WindowOptions {
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("mp4dropXPd — オプション".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(440.), px(580.)),
                    cx,
                ))),
                kind: WindowKind::Normal,
                show: true,
                focus: true,
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    let mut w = OptionsWindow::new(cx);
                    w.recipe = recipe;
                    w.settings = settings;
                    w
                })
            },
        );
    }

    fn open_context_menu(
        &mut self,
        window_origin: gpui::Point<gpui::Pixels>,
        click_pos: gpui::Point<gpui::Pixels>,
        cx: &mut App,
    ) {
        if self.state.encoding {
            return;
        }
        // 既にメニューが開いていれば多重起動を防止する
        if self.menu_open {
            return;
        }

        let recipe = self.state.recipe;
        let settings = self.state.settings.clone();
        let can_encode = !self.state.dropped_paths.is_empty();

        let screen_pos = window_origin + click_pos;
        let bounds = Bounds {
            origin: screen_pos,
            size: size(px(240.), px(480.)),
        };

        let _ = cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                show: true,
                focus: true,
                kind: WindowKind::PopUp,
                is_movable: false,
                is_resizable: false,
                is_minimizable: false,
                window_background: WindowBackgroundAppearance::Opaque,
                ..Default::default()
            },
            |window, cx| {
                // macOS がウィンドウを閉じようとした時に true を返して許可
                window.on_window_should_close(cx, |_, _| true);
                cx.new(|cx| {
                    let mut w = MenuWindow::new(cx);
                    w.recipe = recipe;
                    w.settings = settings;
                    w.can_encode = can_encode;
                    w
                })
            },
        );

        self.menu_open = true;
    }
}

impl Drop for DropWindow {
    fn drop(&mut self) {
        // 残っているエンコードタスクがあれば detach して走り続けさせる
        // (Drop 時には window が既に閉じているため this.update() は失敗するが、let _ = で握り潰される)
        if let Some(task) = self.encode_task.take() {
            task.detach();
        }
    }
}

impl Focusable for DropWindow {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for DropWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_shared(cx);

        let recipe_summary = self.state.recipe.summary();
        let file_count = self.state.dropped_paths.len();
        let encoding = self.state.encoding;
        let progress = self.state.progress;
        // エンコード中の ETA 表示文字列を構築する
        let encoding_label: String = if encoding {
            if progress > 0.001 {
                if let Some(started) = self.state.encode_started {
                    let elapsed = started.elapsed().as_secs_f64();
                    let total_estimated = elapsed / progress;
                    let remaining = (total_estimated - elapsed).max(0.0);
                    format!(
                        "エンコード中… {:.0}% · 残り {:.0}s",
                        progress * 100.0,
                        remaining
                    )
                } else {
                    format!("エンコード中… {:.0}%", progress * 100.0)
                }
            } else {
                "エンコード中…".to_string()
            }
        } else {
            "準備完了".to_string()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x2a2a2a))
            .text_color(rgb(0xe0e0e0))
            .id("root")
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|view, ev: &MouseDownEvent, w, cx| {
                    if view.state.encoding {
                        return;
                    }
                    view.open_context_menu(w.bounds().origin, ev.position, cx);
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, ev: &MouseDownEvent, w, cx| {
                    if ev.modifiers.control && !view.state.encoding {
                        view.open_context_menu(w.bounds().origin, ev.position, cx);
                        return;
                    }
                    if view.menu_open {
                        view.menu_open = false;
                    }
                }),
            )
            // 全体がドロップ領域 + 情報表示 (oggdropXPd 風)
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .p_3()
                    .on_drop(cx.listener(|view, paths: &ExternalPaths, _, cx| {
                        if view.state.encoding {
                            return;
                        }
                        let collected: Vec<PathBuf> =
                            paths.paths().iter().map(|p| p.to_path_buf()).collect();
                        view.state.add_paths(&collected);
                        if view.state.settings.auto_encode_on_drop
                            && !view.state.dropped_paths.is_empty()
                        {
                            view.schedule_encode(cx);
                        }
                    }))
                    // 上部: "MP4 をドロップ"
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .pt_4()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xcccccc))
                                    .child("MP4 をドロップ"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x777777))
                                    .child("右クリックでメニュー · Ctrl+クリック"),
                            ),
                    )
                    // 中央: Audio / Video コーデック情報 + ファイルリスト
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x9aa9c0))
                                    .child(recipe_summary),
                            )
                            .child(div().text_xs().text_color(rgb(0x666666)).child(format!(
                                "音声: {} · 映像: {}",
                                bitrate_label(self.state.settings.audio_bitrate_kbps),
                                bitrate_label(self.state.settings.video_bitrate_kbps),
                            )))
                            .when(file_count > 0, |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x888888))
                                        .child(format!("{file_count} file(s) queued")),
                                )
                            })
                            .children(self.state.dropped_paths.iter().take(3).map(|p| {
                                div().text_xs().text_color(rgb(0x9aa9c0)).child(
                                    p.file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string(),
                                )
                            })),
                    )
                    // 下部: ステータス + プログレスバー風
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .h(px(4.))
                                    .w_full()
                                    .flex()
                                    .flex_row()
                                    .rounded_sm()
                                    .bg(rgb(0x1a1a1a))
                                    .when(encoding && progress > 0.0, |el| {
                                        el.child(
                                            div()
                                                .h_full()
                                                .flex_grow(progress as f32)
                                                .bg(rgb(0x4a6fa5)),
                                        )
                                        .child(div().h_full().flex_grow(1.0 - progress as f32))
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if encoding {
                                                rgb(0xffaa55)
                                            } else {
                                                rgb(0x666666)
                                            })
                                            .child(encoding_label),
                                    ),
                            ),
                    ),
            )
    }
}
