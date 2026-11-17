//! Abrir/fechar a janela do menu e avisos.
//!
//! Ciclo de vida (o que deixa o programa leve parado na bandeja):
//! - a janela NÃO existe até o primeiro uso;
//! - depois de fechar, ela fica pronta por `freeAfterSecs` segundos (reabrir é instantâneo);
//! - passado esse tempo sem uso a janela é destruída e o navegador embutido libera toda a memória.
//!   A próxima abertura recria a janela (uma fração de segundo a mais).

use crate::capture;
use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_notification::NotificationExt;

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window("main").and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

/// Esconde o menu e agenda a liberação da memória.
pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
    schedule_free(app);
}

fn schedule_free(app: &AppHandle) {
    let st = app.state::<AppState>();
    let secs = st.settings.read().unwrap().free_after_secs;
    let gen = st.hide_gen.fetch_add(1, Ordering::SeqCst) + 1;
    if secs == 0 {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(secs));
        let st = app.state::<AppState>();
        if st.hide_gen.load(Ordering::SeqCst) != gen {
            return; // foi aberto de novo nesse meio tempo
        }
        if let Some(w) = app.get_webview_window("main") {
            if !w.is_visible().unwrap_or(false) {
                st.js_ready.store(false, Ordering::SeqCst);
                *st.pending.lock().unwrap() = None;
                let _ = w.destroy();
                st.log.log("tela liberada da memória (parado)");
            }
        }
    });
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

/// Devolve a janela; cria (escondida) se ela foi liberada. O `bool` diz se acabou de ser criada.
fn ensure_window(app: &AppHandle) -> Result<(WebviewWindow, bool), String> {
    if let Some(w) = app.get_webview_window("main") {
        return Ok((w, false));
    }
    let cfg = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .cloned()
        .ok_or("configuração da janela não encontrada")?;
    let w = WebviewWindowBuilder::from_config(app, &cfg)
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;
    Ok((w, true))
}

fn present(app: &AppHandle, win: &WebviewWindow) {
    *app.state::<AppState>().shown_at.lock().unwrap() = Instant::now();
    let _ = win.show();
    let _ = win.set_always_on_top(true);
    let _ = win.set_focus();
}

fn open(app: &AppHandle) -> Result<(), String> {
    let st = app.state::<AppState>();
    st.hide_gen.fetch_add(1, Ordering::SeqCst); // cancela uma liberação de memória agendada
    let t0 = Instant::now();

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

    let (win, created) = ensure_window(app)?;
    if let Some(m) = &monitor {
        let _ = win.set_position(*m.position());
        let _ = win.set_size(*m.size());
    }

    let mut settings = serde_json::to_value(&*st.settings.read().unwrap()).map_err(|e| e.to_string())?;
    settings["packaged"] = json!(true);
    let payload = json!({
        "config": st.store.config_or_empty(),
        "bg": bg,
        "settings": settings,
    });

    if !created && st.js_ready.load(Ordering::SeqCst) {
        present(app, &win);
        win.emit("menu:show", payload).map_err(|e| e.to_string())?;
    } else {
        // janela nova (ou ainda carregando): a tela busca estes dados com `ui_ready` quando ficar pronta
        *st.pending.lock().unwrap() = Some(payload);
    }
    st.log.log(format!(
        "menu aberto (captura em {} ms, total {} ms{})",
        captured.as_millis(),
        t0.elapsed().as_millis(),
        if created { ", janela recriada" } else { "" }
    ));
    Ok(())
}

/// Chamado pela tela assim que carrega. Se há um menu esperando, mostra a janela e devolve os dados.
pub fn ui_ready(app: &AppHandle) -> Option<Value> {
    let st = app.state::<AppState>();
    st.js_ready.store(true, Ordering::SeqCst);
    let pending = st.pending.lock().unwrap().take();
    if pending.is_some() {
        if let Some(w) = app.get_webview_window("main") {
            present(app, &w);
        }
    }
    pending
}
