use gpui::{
    App, Context, FocusHandle, Focusable, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};

use crate::codec::{AudioCodec, EncodeRecipe, VideoCodec};
use crate::settings::AppSettings;

use super::state::{shared, AppCommand};
use super::ui_helpers::{options_section_label, options_row, bitrate_label};
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
    pub(crate) recipe: EncodeRecipe,
    pub(crate) settings: AppSettings,
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
