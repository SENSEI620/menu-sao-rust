//! Gatilho global do mouse: segurar Ctrl + botão esquerdo + botão direito pelo
//! tempo configurado (padrão 1 s) abre o menu. Soltar qualquer um cancela.
//!
//! O Windows chama esta função a CADA movimento do mouse (centenas por segundo) e espera
//! resposta rápida, senão o mouse engasga. Por isso tudo que não é botão/Ctrl sai na primeira linha,
//! sem travar nada, e o gatilho desligado nos ajustes não custa nada.

use crate::state::AppState;
use crate::window;
use rdev::{Button, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};

static ENABLED: AtomicBool = AtomicBool::new(true);

/// Liga/desliga o gesto (ajuste "Abrir segurando os botões").
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

#[derive(Clone, Copy)]
enum Part {
    Left,
    Right,
    Ctrl,
}

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

/// Só interessam botões do mouse e Ctrl; o resto (movimento, rolagem, outras teclas) é descartado.
fn classify(ev: &EventType) -> Option<(Part, bool)> {
    match ev {
        EventType::ButtonPress(Button::Left) => Some((Part::Left, true)),
        EventType::ButtonRelease(Button::Left) => Some((Part::Left, false)),
        EventType::ButtonPress(Button::Right) => Some((Part::Right, true)),
        EventType::ButtonRelease(Button::Right) => Some((Part::Right, false)),
        EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => Some((Part::Ctrl, true)),
        EventType::KeyRelease(Key::ControlLeft | Key::ControlRight) => Some((Part::Ctrl, false)),
        _ => None,
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let st = app.state::<AppState>();
        st.log.log("gatilho do mouse: iniciando");
        let gesture = Arc::new(Mutex::new(Gesture::default()));
        let app2 = app.clone();
        let result = rdev::listen(move |event| {
            let Some((part, down)) = classify(&event.event_type) else { return };
            let (was, now, my_gen);
            {
                let mut g = gesture.lock().unwrap();
                was = g.complete();
                match part {
                    Part::Left => g.left = down,
                    Part::Right => g.right = down,
                    Part::Ctrl => g.ctrl = down,
                }
                now = g.complete();
                if was != now {
                    g.generation += 1;
                }
                my_gen = g.generation;
            }
            // só arma quando acabou de completar (o auto-repetir do Ctrl não reinicia a contagem)
            if now && !was && ENABLED.load(Ordering::Relaxed) {
                arm(&app2, &gesture, my_gen);
            }
        });
        if let Err(e) = result {
            st.log.log(format!("FALHA no gatilho do mouse: {e:?}"));
            window::notify(&app, "Gatilho do mouse indisponível. Use o atalho do teclado.");
        }
    });
}

fn arm(app: &AppHandle, gesture: &Arc<Mutex<Gesture>>, my_gen: u64) {
    let st = app.state::<AppState>();
    let hold = st.settings.read().unwrap().hold_ms;
    if window::is_visible(app) {
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

    #[test]
    fn mouse_moves_and_other_keys_are_ignored_cheaply() {
        assert!(classify(&EventType::MouseMove { x: 1.0, y: 2.0 }).is_none());
        assert!(classify(&EventType::Wheel { delta_x: 0, delta_y: 1 }).is_none());
        assert!(classify(&EventType::KeyPress(Key::KeyA)).is_none());
        assert!(matches!(classify(&EventType::KeyPress(Key::ControlRight)), Some((Part::Ctrl, true))));
        assert!(matches!(classify(&EventType::ButtonRelease(Button::Right)), Some((Part::Right, false))));
    }
}
