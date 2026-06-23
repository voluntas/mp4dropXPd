use std::collections::VecDeque;
use std::sync::Mutex;

use gpui::{App, Global};

use crate::codec::EncodeRecipe;
use crate::settings::AppSettings;

/// 別ウィンドウから DropWindow へのコマンド
pub(crate) enum AppCommand {
    OpenOptions,
    TriggerEncode,
    Clear,
    Exit,
    UpdateRecipe(EncodeRecipe),
    UpdateSettings(AppSettings),
}

#[derive(Default)]
pub(crate) struct SharedState {
    pub(crate) commands: VecDeque<AppCommand>,
}

pub(crate) struct SharedGlobal(pub(crate) Mutex<SharedState>);
impl Global for SharedGlobal {}

pub(crate) fn shared(cx: &App) -> std::sync::MutexGuard<'_, SharedState> {
    cx.global::<SharedGlobal>().0.lock().expect("shared")
}
