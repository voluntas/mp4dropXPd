use gpui::{
    App, Context, FocusHandle, Focusable, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};

use crate::codec::{AudioCodec, EncodeRecipe, VideoCodec};
use crate::settings::AppSettings;

use super::state::{shared, SharedState, AppCommand};
use super::ui_helpers::{menu_sep, menu_label};
pub struct MenuWindow {
    pub(crate) recipe: EncodeRecipe,
    pub(crate) settings: AppSettings,
    pub(crate) can_encode: bool,
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
