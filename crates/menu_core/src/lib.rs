//! Lógica do Menu SAO, separada da janela para poder ser testada sozinha.
//!
//! - `model`:    perfis e links (mesmo formato do config.json da versão Electron)
//! - `settings`: ajustes da engrenagem
//! - `store`:    ler/gravar em %APPDATA%\menu-sao com gravação segura e backup
//! - `browsers`: achar Firefox/Brave e abrir um link no perfil certo
//! - `firefox`:  importar os perfis do Firefox
//! - `log`:      log de diagnóstico (log.txt)

pub mod browsers;
pub mod firefox;
pub mod log;
pub mod model;
pub mod settings;
pub mod store;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Identificador curto e único (7 caracteres), como o `uid()` da versão antiga.
pub fn uid() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut x = nanos ^ COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    // xorshift para espalhar os bits
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    const ALPHA: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut s = String::with_capacity(7);
    for _ in 0..7 {
        s.push(ALPHA[(x % 36) as usize] as char);
        x /= 36;
    }
    s
}

/// Configuração inicial na primeira execução: o perfil principal do Brave, se existir.
pub fn first_run_config(brave_found: bool) -> Vec<model::Profile> {
    let mut cfg = Vec::new();
    if brave_found {
        cfg.push(model::Profile {
            id: uid(),
            name: "Brave — principal".into(),
            icon: "🦁".into(),
            br: "brave".into(),
            pf: "Default".into(),
            ..Default::default()
        });
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uid_is_short_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..5000 {
            let id = uid();
            assert_eq!(id.len(), 7);
            assert!(seen.insert(id));
        }
    }

    #[test]
    fn first_run_has_brave_only_when_found() {
        assert!(first_run_config(false).is_empty());
        let c = first_run_config(true);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].br, "brave");
    }
}
