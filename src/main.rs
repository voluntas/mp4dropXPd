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
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_gui();
}
