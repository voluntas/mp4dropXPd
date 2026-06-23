mod app;
mod codec;
mod encode;
mod error;
mod input;
mod runtime;
mod settings;

use app::run_gui;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    run_gui();
}
