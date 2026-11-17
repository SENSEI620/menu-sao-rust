//! Atalho global do teclado (padrão Ctrl+Alt+M, mudável na engrenagem).

use crate::state::AppState;
use crate::window;
use menu_core::settings::Settings;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn apply(app: &AppHandle, s: &Settings) {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    if s.shortcut.is_empty() {
        return;
    }
    let handle = app.clone();
    let result = gs.on_shortcut(s.shortcut.as_str(), move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            window::show_menu(&handle);
        }
    });
    let log = &app.state::<AppState>().log;
    match result {
        Ok(()) => log.log(format!("atalho {} registrado", s.shortcut)),
        Err(e) => {
            log.log(format!("atalho {} NAO registrado: {e}", s.shortcut));
            window::notify(app, format!("Não consegui usar o atalho {} (outro programa já usa). Mude na engrenagem.", s.shortcut));
        }
    }
}
