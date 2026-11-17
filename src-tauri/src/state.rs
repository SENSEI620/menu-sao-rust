use menu_core::log::Logger;
use menu_core::settings::Settings;
use menu_core::store::Store;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};
use std::time::Instant;

/// Tudo que o programa compartilha entre a janela, a bandeja e o gesto do mouse.
pub struct AppState {
    pub store: Store,
    pub log: Logger,
    pub settings: RwLock<Settings>,
    /// quando o menu foi aberto (a janela não fecha por perda de foco nos primeiros 700 ms)
    pub shown_at: Mutex<Instant>,
    /// evita abrir o menu duas vezes ao mesmo tempo
    pub opening: AtomicBool,
}
