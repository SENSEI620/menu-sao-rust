//! Abrir/fechar a janela do menu e avisos.

use crate::capture;
use crate::state::AppState;
use serde_json::json;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

/// Aviso: dentro do menu (se estiver aberto) ou como notificação do Windows.
pub fn notify(app: &AppHandle, msg: impl Into<String>) {
    let msg = msg.into();
    if is_visible(app) {
        let _ = app.emit("toast", &msg);
    } else {
        let _ = app.notification().builder().title("Menu SAO").body(&msg).show();
    }
}

/// Abre o menu na tela onde o mouse está. Roda em outra thread, para não travar nada.
pub fn show_menu(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let st = app.state::<AppState>();
        if is_visible(&app) || st.opening.swap(true, Ordering::SeqCst) {
            return;
        }
        let result = open(&app);
        st.opening.store(false, Ordering::SeqCst);
        if let Err(e) = result {
            st.log.log(format!("não consegui abrir o menu: {e}"));
        }
    });
}

fn open(app: &AppHandle) -> Result<(), String> {
    let st = app.state::<AppState>();
    let t0 = Instant::now();
    let win = app.get_webview_window("main").ok_or("janela não encontrada")?;

    let cursor = app.cursor_position().map_err(|e| e.to_string())?;
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());

    let bg = match capture::monitor_at(cursor.x as i32, cursor.y as i32) {
        Ok(shot) => {
            st.log.log(format!("captura da tela: {}x{}", shot.width, shot.height));
            Some(shot.data_url)
        }
        Err(e) => {
            st.log.log(format!("captura de tela falhou: {e}"));
            None
        }
    };
    let captured = t0.elapsed();

    if let Some(m) = &monitor {
        let _ = win.set_position(*m.position());
        let _ = win.set_size(*m.size());
    }
    *st.shown_at.lock().unwrap() = Instant::now();
    let _ = win.show();
    let _ = win.set_always_on_top(true);
    let _ = win.set_focus();

    let mut settings = serde_json::to_value(&*st.settings.read().unwrap()).map_err(|e| e.to_string())?;
    settings["packaged"] = json!(true);
    let payload = json!({
        "config": st.store.config_or_empty(),
        "bg": bg,
        "settings": settings,
    });
    win.emit("menu:show", payload).map_err(|e| e.to_string())?;
    st.log.log(format!("menu aberto (captura da tela em {} ms)", captured.as_millis()));
    Ok(())
}
