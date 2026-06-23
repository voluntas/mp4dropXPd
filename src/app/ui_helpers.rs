use gpui::{div, rgb, IntoElement, Styled, prelude::*};

pub(crate) fn menu_sep() -> gpui::Div {
    div().h_px().bg(rgb(0x444444)).my_1()
}

pub(crate) fn menu_label(label: &'static str) -> gpui::Div {
    div()
        .px_3()
        .py_0p5()
        .text_xs()
        .text_color(rgb(0x888888))
        .child(label)
}

/// ビットレート表示用ラベル
/// 0 のときは「Auto」(入力の実ビットレートを引き継ぐことを示す)
pub(crate) fn bitrate_label(kbps: u32) -> String {
    if kbps == 0 {
        "自動".to_string()
    } else {
        format!("{kbps} kbps")
    }
}

pub(crate) fn options_section_label(label: &'static str) -> gpui::Div {
    div()
        .text_xs()
        .text_color(rgb(0x888888))
        .mt_2()
        .mb_1()
        .child(label.to_uppercase())
}

pub(crate) fn options_row(label: &'static str, control: impl IntoElement) -> gpui::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .py_1()
        .child(div().text_sm().text_color(rgb(0xe0e0e0)).child(label))
        .child(control)
}
