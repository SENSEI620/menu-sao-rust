//! Gatilho global do mouse: segurar Ctrl + botão esquerdo + botão direito pelo
//! tempo configurado (padrão 1 s) abre o menu. Soltar qualquer um cancela.

use crate::state::AppState;
use crate::window;
use rdev::{Button, EventType, Key};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};

#[derive(Default)]
struct Gesture {
    left: bool,
    right: bool,
    ctrl: bool,
    /// cada mudança de estado aumenta; o temporizador só vale se a geração não mudou
    generation: u64,
}

impl Gesture {
    fn complete(&self) -> bool {
        self.left && self.right && self.ctrl
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let st = app.state::<AppState>();
        st.log.log("gatilho do mouse: iniciando");
        let gesture = Arc::new(Mutex::new(Gesture::default()));
        let app2 = app.clone();
        let result = rdev::listen(move |event| {
            let was;
            let now;
            let my_gen;
            {
                let mut g = gesture.lock().unwrap();
                was = g.complete();
                match event.event_type {
                    EventType::ButtonPress(Button::Left) => g.left = true,
                    EventType::ButtonRelease(Button::Left) => g.left = false,
                    EventType::ButtonPress(Button::Right) => g.right = true,
                    EventType::ButtonRelease(Button::Right) => g.right = false,
                    EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => g.ctrl = true,
                    EventType::KeyRelease(Key::ControlLeft | Key::ControlRight) => g.ctrl = false,
                    _ => return,
                }
                now = g.complete();
                if was != now {
                    g.generation += 1;
                }
                my_gen = g.generation;
            }
            // só arma quando acabou de completar (o auto-repetir do Ctrl não reinicia a contagem)
            if now && !was {
                arm(&app2, &gesture, my_gen);
            }
        });
        if let Err(e) = result {
            st.log.log(format!("FALHA no gatilho do mouse: {e:?}"));
            window::notify(&app, format!("Gatilho do mouse indisponível. Use o atalho do teclado."));
        }
    });
}

fn arm(app: &AppHandle, gesture: &Arc<Mutex<Gesture>>, my_gen: u64) {
    let st = app.state::<AppState>();
    let (enabled, hold) = {
        let s = st.settings.read().unwrap();
        (s.mouse_gesture, s.hold_ms)
    };
    if !enabled || window::is_visible(app) {
        return;
    }
    st.log.log("gatilho armado");
    let app = app.clone();
    let gesture = Arc::clone(gesture);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(hold));
        let still = {
            let g = gesture.lock().unwrap();
            g.generation == my_gen && g.complete()
        };
        if still {
            app.state::<AppState>().log.log("gatilho completo");
            window::show_menu(&app);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_needs_all_three() {
        let mut g = Gesture::default();
        assert!(!g.complete());
        g.left = true;
        g.right = true;
        assert!(!g.complete());
        g.ctrl = true;
        assert!(g.complete());
    }
}
