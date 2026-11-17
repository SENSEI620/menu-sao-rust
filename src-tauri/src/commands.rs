//! Comandos que a tela do menu pode chamar.

use crate::shortcuts;
use crate::state::AppState;
use crate::window;
use menu_core::browsers::{self, Browser};
use menu_core::firefox;
use menu_core::model::Profile;
use menu_core::settings::Settings;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub fn save_config(state: State<'_, AppState>, config: Vec<Profile>) {
    if let Err(e) = state.store.save_config(&config) {
        state.log.log(format!("erro ao salvar config: {e}"));
    }
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: Value) {
    let s = Settings::from_json(&settings);
    if let Err(e) = state.store.save_settings(&s) {
        state.log.log(format!("erro ao salvar ajustes: {e}"));
    }
    *state.settings.write().unwrap() = s.clone();
    crate::hook::set_enabled(s.mouse_gesture);
    shortcuts::apply(&app, &s);
    apply_autostart(&app, s.autostart);
}

/// Liga/desliga "Iniciar com o Windows" só se mudou.
pub fn apply_autostart(app: &AppHandle, want: bool) {
    let m = app.autolaunch();
    if m.is_enabled().unwrap_or(false) != want {
        let r = if want { m.enable() } else { m.disable() };
        if let Err(e) = r {
            app.state::<AppState>().log.log(format!("início automático: {e}"));
        }
    }
}

#[tauri::command]
pub async fn import_firefox(state: State<'_, AppState>) -> Result<Value, String> {
    let mut cfg = state.store.config_or_empty();
    let n = firefox::import(&mut cfg, &firefox::firefox_root(), &|m| state.log.log(m));
    state.store.save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(json!({ "n": n, "config": cfg }))
}

/// A tela avisa que carregou; se há um menu esperando, a janela é mostrada e os dados voltam aqui.
#[tauri::command]
pub fn ui_ready(app: AppHandle) -> Option<Value> {
    window::ui_ready(&app)
}

#[tauri::command]
pub fn hide_menu(app: AppHandle) {
    window::hide(&app);
}

#[tauri::command]
pub fn launch(app: AppHandle, state: State<'_, AppState>, br: String, pf: String, url: String) {
    let no_remote = state.settings.read().unwrap().firefox_no_remote;
    match browsers::launch(Browser::parse(&br), &pf, &url, no_remote) {
        Ok((exe, args)) => state.log.log(format!("abrindo {} {}", exe.display(), args.join(" "))),
        Err(e) => {
            state.log.log(format!("erro ao abrir: {e:?}"));
            window::notify(&app, e.to_string());
        }
    }
}
