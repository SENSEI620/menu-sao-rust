// Sem janela de console no Windows (o programa roda em segundo plano).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod commands;
mod hook;
mod shortcuts;
mod state;
mod tray;
mod window;

use menu_core::browsers::{self, Browser};
use menu_core::log::Logger;
use menu_core::store::{Loaded, Store};
use menu_core::{first_run_config, firefox};
use state::AppState;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};
use tauri::{Manager, RunEvent, WindowEvent};

fn main() {
    let app = tauri::Builder::default()
        // segunda execução: em vez de abrir outro programa, abre o menu desta cópia
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(st) = app.try_state::<AppState>() {
                st.log.log("segunda execução: abrindo o menu");
            }
            window::show_menu(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![
            commands::save_config,
            commands::save_settings,
            commands::import_firefox,
            commands::hide_menu,
            commands::ui_ready,
            commands::launch,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let store = Store::new(Store::default_dir());
            let log = Logger::new(store.log_path());
            log.log(format!("programa iniciado (Rust + Tauri {})", tauri::VERSION));

            match store.load_config() {
                Loaded::Missing => {
                    log.log("primeira execução: criando a configuração");
                    let mut cfg = first_run_config(browsers::find_exe(Browser::Brave).is_some());
                    firefox::import(&mut cfg, &firefox::firefox_root(), &|m| log.log(m));
                    let _ = store.save_config(&cfg);
                }
                Loaded::Corrupt { saved_as } => {
                    log.log(format!("config.json corrompido; guardado em {}", saved_as.display()));
                }
                Loaded::Ok(_) => {}
            }

            let settings = store.load_settings();
            app.manage(AppState {
                store,
                log,
                settings: RwLock::new(settings.clone()),
                shown_at: Mutex::new(Instant::now()),
                opening: AtomicBool::new(false),
                js_ready: AtomicBool::new(false),
                pending: Mutex::new(None),
                hide_gen: AtomicU64::new(0),
            });

            tray::setup(&handle)?;
            shortcuts::apply(&handle, &settings);
            commands::apply_autostart(&handle, settings.autostart);
            hook::set_enabled(settings.mouse_gesture);
            hook::start(handle);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // fecha sozinho ao perder o foco (depois dos primeiros 700 ms)
            WindowEvent::Focused(false) => {
                let recent = window
                    .app_handle()
                    .try_state::<AppState>()
                    .map(|s| s.shown_at.lock().unwrap().elapsed() < Duration::from_millis(700))
                    .unwrap_or(true);
                if !recent && window.is_visible().unwrap_or(false) {
                    window::hide(window.app_handle());
                }
            }
            // o programa continua na bandeja: "fechar" só esconde
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                window::hide(window.app_handle());
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("erro ao iniciar o Menu SAO");

    app.run(|_app, event| {
        if let RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none() {
                api.prevent_exit(); // sem janela visível o programa fica rodando na bandeja
            }
        }
    });
}
