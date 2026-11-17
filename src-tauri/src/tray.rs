//! Ícone na bandeja (perto do relógio).

use crate::state::AppState;
use crate::{commands, window};
use menu_core::firefox;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir menu", true, None::<&str>)?;
    let log = MenuItem::with_id(app, "log", "Ver log (diagnóstico)", true, None::<&str>)?;
    let import = MenuItem::with_id(app, "import", "Importar perfis do Firefox", true, None::<&str>)?;
    let auto_on = app.autolaunch().is_enabled().unwrap_or(false);
    let auto = CheckMenuItem::with_id(app, "autostart", "Iniciar com o Windows", true, auto_on, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &log, &import, &auto, &sep, &quit])?;

    let auto_item = auto.clone();
    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Menu SAO")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "open" => window::show_menu(app),
            "log" => {
                let _ = opener::open(app.state::<AppState>().store.log_path());
            }
            "import" => {
                let st = app.state::<AppState>();
                let mut cfg = st.store.config_or_empty();
                let n = firefox::import(&mut cfg, &firefox::firefox_root(), &|m| st.log.log(m));
                let _ = st.store.save_config(&cfg);
                window::notify(app, format!("{n} perfil(is) importado(s) ou corrigido(s)."));
            }
            "autostart" => {
                let want = auto_item.is_checked().unwrap_or(false);
                let st = app.state::<AppState>();
                let mut s = st.settings.write().unwrap();
                s.autostart = want;
                let _ = st.store.save_settings(&s);
                drop(s);
                commands::apply_autostart(app, want);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                window::show_menu(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}
