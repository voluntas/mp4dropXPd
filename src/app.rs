use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use gpui::{
    App, AppContext, Bounds, Context, ExternalPaths, FocusHandle, Focusable, Global, MouseButton,
    MouseDownEvent, SharedString, Styled, Task, Window, WindowBackgroundAppearance, WindowBounds,
    WindowKind, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use gpui_tokio::Tokio;

use crate::codec::{AudioCodec, EncodeRecipe, VideoCodec};
use crate::encode::{self, EncodeJob, JobProgress, default_output_path};
use crate::input::filter_mp4_paths;
use crate::settings::AppSettings;

/// 別ウィンドウから DropWindow へのコマンド
enum AppCommand {
    OpenOptions,
    TriggerEncode,
    Clear,
    Exit,
    UpdateRecipe(EncodeRecipe),
    UpdateSettings(AppSettings),
}

#[derive(Default)]
struct SharedState {
    commands: VecDeque<AppCommand>,
}

struct SharedGlobal(Mutex<SharedState>);
impl Global for SharedGlobal {}

fn shared(cx: &App) -> std::sync::MutexGuard<'_, SharedState> {
    cx.global::<SharedGlobal>().0.lock().expect("shared")
}

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
    fn new(cx: &mut Context<Self>) -> Self {
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

// ===== MenuWindow (別ウィンドウ) =====

pub struct MenuWindow {
    recipe: EncodeRecipe,
    settings: AppSettings,
    can_encode: bool,
    focus: FocusHandle,
}

impl MenuWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        Self {
            recipe: EncodeRecipe::default(),
            settings: AppSettings::default(),
            can_encode: false,
            focus,
        }
    }

    fn publish_recipe(&self, cx: &mut App) {
        let mut s = shared(cx);
        s.commands.push_back(AppCommand::UpdateRecipe(self.recipe));
    }

    fn publish_settings(&self, cx: &mut App) {
        let mut s = shared(cx);
        s.commands.push_back(AppCommand::UpdateSettings(self.settings.clone()));
    }

    fn request(&self, cx: &mut App, f: impl FnOnce(&mut SharedState)) {
        let mut s = shared(cx);
        f(&mut s);
    }
}

impl Focusable for MenuWindow {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for MenuWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // メニュー項目クリック時は listener クロージャで直接 _w.remove_window() を呼ぶため
        // pending_close_menu は不要 (0022 で削除)

        div()
            .flex()
            .flex_col()
            .min_w(px(240.))
            .py_1()
            .bg(rgb(0x2d2d2d))
            .border_1()
            .border_color(rgb(0x555555))
            .rounded_md()
            .shadow_lg()
            .child(
                div()
                    .id("m-options")
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(rgb(0xe0e0e0))
                    .hover(|s| s.bg(rgb(0x404040)))
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _ev, _w, cx| {
                        view.request(cx, |s| s.commands.push_back(AppCommand::OpenOptions));
                        _w.remove_window();
                    }))
                    .child("オプション…"),
            )
            .child(menu_sep())
            .child(menu_label("音声"))
            .children(AudioCodec::ALL.into_iter().map(|a| {
                let sel = a == self.recipe.audio;
                div()
                    .id(format!("m-a-{}", a.label()))
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(rgb(0xe0e0e0))
                    .bg(if sel { rgb(0x4a6fa5) } else { rgb(0x2d2d2d) })
                    .hover(|s| s.bg(rgb(0x404040)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |view, _ev, _w, cx| {
                        view.recipe.audio = a;
                        view.publish_recipe(cx);
                        cx.notify();
                    }))
                    .child(if sel {
                        format!("● {}  ", a.label())
                    } else {
                        format!("    {}", a.label())
                    })
            }))
            .child(menu_sep())
            .child(menu_label("映像"))
            .children(VideoCodec::ALL.into_iter().map(|v| {
                let sel = v == self.recipe.video;
                div()
                    .id(format!("m-v-{}", v.label()))
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(rgb(0xe0e0e0))
                    .bg(if sel { rgb(0x4a6fa5) } else { rgb(0x2d2d2d) })
                    .hover(|s| s.bg(rgb(0x404040)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |view, _ev, _w, cx| {
                        view.recipe.video = v;
                        view.publish_recipe(cx);
                        cx.notify();
                    }))
                    .child(if sel {
                        format!("● {}  ", v.label())
                    } else {
                        format!("    {}", v.label())
                    })
            }))
            .child(menu_sep())
            .child({
                let checked = self.settings.auto_encode_on_drop;
                div()
                    .id("m-toggle-auto")
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(rgb(0xe0e0e0))
                    .hover(|s| s.bg(rgb(0x404040)))
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _ev, _w, cx| {
                        view.settings.auto_encode_on_drop = !view.settings.auto_encode_on_drop;
                        view.publish_settings(cx);
                        cx.notify();
                    }))
                    .child(if checked {
                        "● ドロップ時自動エンコード".to_string()
                    } else {
                        "    ドロップ時自動エンコード".to_string()
                    })
            })
            .child({
                let checked = self.settings.overwrite;
                div()
                    .id("m-toggle-overwrite")
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(rgb(0xe0e0e0))
                    .hover(|s| s.bg(rgb(0x404040)))
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _ev, _w, cx| {
                        view.settings.overwrite = !view.settings.overwrite;
                        view.publish_settings(cx);
                        cx.notify();
                    }))
                    .child(if checked {
                        "● Overwrite output".to_string()
                    } else {
                        "    Overwrite output".to_string()
                    })
            })
            .child(menu_sep())
            .child({
                let enabled = self.can_encode;
                let label: SharedString = if enabled {
                    "エンコード".into()
                } else {
                    "エンコード (ファイルなし)".into()
                };
                let mut base =
                    div()
                        .id("m-encode")
                        .px_3()
                        .py_1()
                        .text_sm()
                        .text_color(if enabled {
                            rgb(0xe0e0e0)
                        } else {
                            rgb(0x666666)
                        });
                if enabled {
                    base = base
                        .hover(|s| s.bg(rgb(0x404040)))
                        .cursor_pointer()
                        .on_click(cx.listener(|view, _ev, _w, cx| {
                            view.request(cx, |s| s.commands.push_back(AppCommand::TriggerEncode));
                            _w.remove_window();
                        }));
                } else {
                    base = base.opacity(0.5);
                }
                base.child(label)
            })
            .child(
                div()
                    .id("m-clear")
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(rgb(0xe0e0e0))
                    .hover(|s| s.bg(rgb(0x404040)))
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _ev, _w, cx| {
                        view.request(cx, |s| s.commands.push_back(AppCommand::Clear));
                        _w.remove_window();
                    }))
                    .child("クリア"),
            )
            .child(menu_sep())
            .child(
                div()
                    .id("m-exit")
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(rgb(0xe0e0e0))
                    .hover(|s| s.bg(rgb(0x404040)))
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _ev, _w, cx| {
                        view.request(cx, |s| s.commands.push_back(AppCommand::Exit));
                        _w.remove_window();
                    }))
                    .child("終了"),
            )
            .into_any_element()
    }
}

fn menu_sep() -> gpui::Div {
    div().h_px().bg(rgb(0x444444)).my_1()
}

fn menu_label(label: &'static str) -> gpui::Div {
    div()
        .px_3()
        .py_0p5()
        .text_xs()
        .text_color(rgb(0x888888))
        .child(label)
}

// ===== Options Window =====

/// ビットレート表示用ラベル
/// 0 のときは「Auto」(入力の実ビットレートを引き継ぐことを示す)
fn bitrate_label(kbps: u32) -> String {
    if kbps == 0 {
        "自動".to_string()
    } else {
        format!("{kbps} kbps")
    }
}

/// 映像ビットレートのプリセット (kbps)
const VIDEO_BITRATE_PRESETS: [u32; 6] = [500, 1000, 2000, 4000, 8000, 16000];
/// 映像ビットレートのステップ幅 (kbps)
const VIDEO_BITRATE_STEP: u32 = 100;
/// 映像ビットレートの最小値 (kbps)
const VIDEO_BITRATE_MIN: u32 = 100;
/// 映像ビットレートの最大値 (kbps)
const VIDEO_BITRATE_MAX: u32 = 50_000;

/// 音声ビットレートのプリセット (kbps)
const AUDIO_BITRATE_PRESETS: [u32; 6] = [64, 96, 128, 192, 256, 320];
/// 音声ビットレートのステップ幅 (kbps)
const AUDIO_BITRATE_STEP: u32 = 16;
/// 音声ビットレートの最小値 (kbps)
const AUDIO_BITRATE_MIN: u32 = 16;
/// 音声ビットレートの最大値 (kbps)
const AUDIO_BITRATE_MAX: u32 = 512;

pub struct OptionsWindow {
    recipe: EncodeRecipe,
    settings: AppSettings,
    status: SharedString,
    focus: FocusHandle,
}

impl OptionsWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        Self {
            recipe: EncodeRecipe::default(),
            settings: AppSettings::default(),
            status: "保存ボタンで反映".into(),
            focus,
        }
    }

    /// 現在の recipe をメインウィンドウへ反映させる
    fn publish_recipe(&self, cx: &mut App) {
        let mut s = shared(cx);
        s.commands.push_back(AppCommand::UpdateRecipe(self.recipe));
    }

    /// 現在の settings をメインウィンドウへ反映させる
    fn publish_settings(&self, cx: &mut App) {
        let mut s = shared(cx);
        s.commands.push_back(AppCommand::UpdateSettings(self.settings.clone()));
    }
}

impl Focusable for OptionsWindow {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

fn options_section_label(label: &'static str) -> gpui::Div {
    div()
        .text_xs()
        .text_color(rgb(0x888888))
        .mt_2()
        .mb_1()
        .child(label.to_uppercase())
}

fn options_row(label: &'static str, control: impl IntoElement) -> gpui::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .py_1()
        .child(div().text_sm().text_color(rgb(0xe0e0e0)).child(label))
        .child(control)
}

impl Render for OptionsWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .gap_2()
            .bg(rgb(0x2a2a2a))
            .text_color(rgb(0xe0e0e0))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("mp4dropXPd — オプション"),
            )
            .child(options_section_label("音声コーデック"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .children(AudioCodec::ALL.into_iter().map(|a| {
                        let sel = a == self.recipe.audio;
                        div()
                            .id(a.label())
                            .px_2()
                            .py_0p5()
                            .text_xs()
                            .rounded_sm()
                            .cursor_pointer()
                            .when(sel, |el| el.bg(rgb(0x4a6fa5)))
                            .when(!sel, |el| el.bg(rgb(0x3a3a3a)))
                            .on_click(cx.listener(move |view, _, _, _| {
                                view.recipe.audio = a;
                            }))
                            .child(a.label())
                    })),
            )
            .child(options_section_label("映像コーデック"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .children(VideoCodec::ALL.into_iter().map(|v| {
                        let sel = v == self.recipe.video;
                        div()
                            .id(v.label())
                            .px_2()
                            .py_0p5()
                            .text_xs()
                            .rounded_sm()
                            .cursor_pointer()
                            .when(sel, |el| el.bg(rgb(0x4a6fa5)))
                            .when(!sel, |el| el.bg(rgb(0x3a3a3a)))
                            .on_click(cx.listener(move |view, _, _, _| {
                                view.recipe.video = v;
                            }))
                            .child(v.label())
                    })),
            )
            .child(options_section_label("動作"))
            .child(options_row(
                "ドロップ時自動エンコード",
                div()
                    .id("opt-auto-encode")
                    .px_2()
                    .py_0p5()
                    .text_xs()
                    .rounded_sm()
                    .bg(rgb(if self.settings.auto_encode_on_drop {
                        0x4a6fa5
                    } else {
                        0x3a3a3a
                    }))
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _, _, _| {
                        view.settings.auto_encode_on_drop = !view.settings.auto_encode_on_drop;
                    }))
                    .child(if self.settings.auto_encode_on_drop {
                        "オン"
                    } else {
                        "オフ"
                    }),
            ))
            .child(options_row(
                "Overwrite output",
                div()
                    .id("opt-overwrite")
                    .px_2()
                    .py_0p5()
                    .text_xs()
                    .rounded_sm()
                    .bg(rgb(if self.settings.overwrite {
                        0x4a6fa5
                    } else {
                        0x3a3a3a
                    }))
                    .cursor_pointer()
                    .on_click(cx.listener(|view, _, _, _| {
                        view.settings.overwrite = !view.settings.overwrite;
                    }))
                    .child(if self.settings.overwrite { "オン" } else { "オフ" }),
            ))
            // ===== 音声ビットレート =====
            .child(options_section_label("音声ビットレート"))
            // プリセット選択
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .children(AUDIO_BITRATE_PRESETS.iter().map(|&preset| {
                        let sel = self.settings.audio_bitrate_kbps == preset;
                        div()
                            .id(format!("a-preset-{preset}"))
                            .px_1()
                            .py_0p5()
                            .text_xs()
                            .rounded_sm()
                            .cursor_pointer()
                            .when(sel, |el| el.bg(rgb(0x4a6fa5)))
                            .when(!sel, |el| el.bg(rgb(0x3a3a3a)))
                            .on_click(cx.listener(move |view, _, _, cx| {
                                view.settings.audio_bitrate_kbps = preset;
                                cx.notify();
                            }))
                            .child(format!("{preset}"))
                    })),
            )
            // ステッパ (- 値 +)
            .child(options_row(
                "音声 (kbps)",
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .id("a-dec")
                            .px_2()
                            .py_0p5()
                            .text_xs()
                            .rounded_sm()
                            .bg(rgb(0x4a4a4a))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x5a5a5a)))
                            .on_click(cx.listener(|view, _, _, cx| {
                                let v = view.settings.audio_bitrate_kbps;
                                view.settings.audio_bitrate_kbps = if v == 0 {
                                    AUDIO_BITRATE_MIN
                                } else {
                                    v.saturating_sub(AUDIO_BITRATE_STEP).max(AUDIO_BITRATE_MIN)
                                };
                                cx.notify();
                            }))
                            .child("-"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xb0b0b0))
                            .w(px(80.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(bitrate_label(self.settings.audio_bitrate_kbps).to_string()),
                    )
                    .child(
                        div()
                            .id("a-inc")
                            .px_2()
                            .py_0p5()
                            .text_xs()
                            .rounded_sm()
                            .bg(rgb(0x4a4a4a))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x5a5a5a)))
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.settings.audio_bitrate_kbps =
                                    if view.settings.audio_bitrate_kbps == 0 {
                                        AUDIO_BITRATE_MIN
                                    } else {
                                        view.settings
                                            .audio_bitrate_kbps
                                            .saturating_add(AUDIO_BITRATE_STEP)
                                            .min(AUDIO_BITRATE_MAX)
                                    };
                                cx.notify();
                            }))
                            .child("+"),
                    ),
            ))
            // ===== 映像ビットレート =====
            .child(options_section_label("映像ビットレート"))
            // プリセット選択
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .children(VIDEO_BITRATE_PRESETS.iter().map(|&preset| {
                        let sel = self.settings.video_bitrate_kbps == preset;
                        div()
                            .id(format!("v-preset-{preset}"))
                            .px_1()
                            .py_0p5()
                            .text_xs()
                            .rounded_sm()
                            .cursor_pointer()
                            .when(sel, |el| el.bg(rgb(0x4a6fa5)))
                            .when(!sel, |el| el.bg(rgb(0x3a3a3a)))
                            .on_click(cx.listener(move |view, _, _, cx| {
                                view.settings.video_bitrate_kbps = preset;
                                cx.notify();
                            }))
                            .child(format!("{preset}"))
                    })),
            )
            // ステッパ (- 値 +)
            .child(options_row(
                "映像 (kbps)",
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .id("v-dec")
                            .px_2()
                            .py_0p5()
                            .text_xs()
                            .rounded_sm()
                            .bg(rgb(0x4a4a4a))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x5a5a5a)))
                            .on_click(cx.listener(|view, _, _, cx| {
                                let v = view.settings.video_bitrate_kbps;
                                view.settings.video_bitrate_kbps = if v == 0 {
                                    VIDEO_BITRATE_MIN
                                } else {
                                    v.saturating_sub(VIDEO_BITRATE_STEP).max(VIDEO_BITRATE_MIN)
                                };
                                cx.notify();
                            }))
                            .child("-"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xb0b0b0))
                            .w(px(80.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(bitrate_label(self.settings.video_bitrate_kbps).to_string()),
                    )
                    .child(
                        div()
                            .id("v-inc")
                            .px_2()
                            .py_0p5()
                            .text_xs()
                            .rounded_sm()
                            .bg(rgb(0x4a4a4a))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x5a5a5a)))
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.settings.video_bitrate_kbps =
                                    if view.settings.video_bitrate_kbps == 0 {
                                        VIDEO_BITRATE_MIN
                                    } else {
                                        view.settings
                                            .video_bitrate_kbps
                                            .saturating_add(VIDEO_BITRATE_STEP)
                                            .min(VIDEO_BITRATE_MAX)
                                    };
                                cx.notify();
                            }))
                            .child("+"),
                    ),
            ))
            .child(div().flex_grow(1.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .gap_2()
                    .child(
                        div()
                            .id("opt-close")
                            .px_3()
                            .py_1()
                            .text_xs()
                            .rounded_sm()
                            .bg(rgb(0x4a4a4a))
                            .cursor_pointer()
                            .on_click(cx.listener(|_, _, w, _cx| {
                                w.remove_window();
                            }))
                            .child("閉じる"),
                    )
                    .child(
                        div()
                            .id("opt-save")
                            .px_3()
                            .py_1()
                            .text_xs()
                            .rounded_sm()
                            .bg(rgb(0x3d7a3d))
                            .cursor_pointer()
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.publish_recipe(cx);
                                view.publish_settings(cx);
                                view.status = "保存しました".into();
                                cx.notify();
                            }))
                            .child("保存"),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(self.status.clone()),
            )
    }
}

pub fn run_gui() {
    application().run(|cx: &mut App| {
        crate::runtime::init_gpui(cx);
        cx.set_global(SharedGlobal(Mutex::new(SharedState::default())));

        let bounds = Bounds::centered(None, size(px(240.), px(240.)), cx);
        cx.open_window(
            WindowOptions {
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("mp4dropXPd".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                kind: WindowKind::Normal,
                is_movable: true,
                is_resizable: false,
                window_background: WindowBackgroundAppearance::Opaque,
                show: true,
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(DropWindow::new),
        )
        .expect("failed to open window");
        cx.activate(true);
    });
}
