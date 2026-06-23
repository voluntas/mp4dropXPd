mod drop_window;
mod menu_window;
mod options_window;
mod state;
mod ui_helpers;

use gpui::{
    AppContext, Bounds, TitlebarOptions, WindowBackgroundAppearance,
    WindowBounds, WindowKind, WindowOptions, px, size,
};

use crate::runtime;

pub use drop_window::DropWindow;
pub(crate) use state::SharedGlobal;

pub fn run_gui() {
    let app = gpui_platform::application();
    app
        .with_assets(())
        .run(move |cx| {
            runtime::init_gpui(cx);
            cx.set_global(SharedGlobal(Default::default()));
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("mp4dropXPd".into()),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(380.), px(230.)),
                        cx,
                    ))),
                    kind: WindowKind::Normal,
                    window_background: WindowBackgroundAppearance::Opaque,
                    show: true,
                    focus: true,
                    ..Default::default()
                },
                |_, cx| cx.new(DropWindow::new),
            )
            .expect("open main window");

            cx.activate(true);
        });
}
