//! Ajustes da engrenagem (settings.json, mesmos nomes de campo da versão Electron).

use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// atalho global, ex.: "Control+Alt+M" (vazio = sem atalho)
    pub shortcut: String,
    /// abrir segurando Ctrl + botão esquerdo + botão direito
    pub mouse_gesture: bool,
    /// quanto tempo segurar, em milissegundos (300 a 5000)
    pub hold_ms: u64,
    /// volume dos sons (0 a 100)
    pub volume: u8,
    /// Firefox: abrir em janela separada (-no-remote)
    pub firefox_no_remote: bool,
    /// iniciar com o Windows
    pub autostart: bool,
    /// segundos parado até liberar a memória da tela (0 = nunca liberar)
    pub free_after_secs: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            shortcut: "Control+Alt+M".into(),
            mouse_gesture: true,
            hold_ms: 1000,
            volume: 100,
            firefox_no_remote: false,
            autostart: false,
            free_after_secs: 60,
        }
    }
}

impl Settings {
    /// Monta os ajustes a partir de um JSON qualquer, usando o padrão onde faltar
    /// e limitando os valores (como a versão antiga fazia).
    pub fn from_json(v: &Value) -> Settings {
        let d = Settings::default();
        let num = |k: &str| v.get(k).and_then(Value::as_f64);
        let flag = |k: &str, def: bool| v.get(k).and_then(Value::as_bool).unwrap_or(def);
        Settings {
            shortcut: match v.get("shortcut") {
                Some(Value::String(s)) => s.trim().to_string(),
                Some(Value::Null) => String::new(),
                _ => d.shortcut,
            },
            mouse_gesture: flag("mouseGesture", d.mouse_gesture),
            hold_ms: num("holdMs").filter(|n| n.is_finite() && *n > 0.0).map(|n| n.round() as u64).unwrap_or(d.hold_ms).clamp(300, 5000),
            volume: num("volume").filter(|n| n.is_finite()).map(|n| n.round().clamp(0.0, 100.0) as u8).unwrap_or(d.volume),
            firefox_no_remote: flag("firefoxNoRemote", d.firefox_no_remote),
            autostart: flag("autostart", d.autostart),
            free_after_secs: num("freeAfterSecs").filter(|n| n.is_finite() && *n >= 0.0).map(|n| n.round() as u64).unwrap_or(d.free_after_secs).min(3600),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults_when_empty() {
        assert_eq!(Settings::from_json(&json!({})), Settings::default());
    }

    #[test]
    fn clamps_values() {
        let s = Settings::from_json(&json!({"holdMs": 99999, "volume": 500}));
        assert_eq!((s.hold_ms, s.volume), (5000, 100));
        let s = Settings::from_json(&json!({"holdMs": 10, "volume": -3}));
        assert_eq!((s.hold_ms, s.volume), (300, 0));
    }

    #[test]
    fn free_after_secs_default_clamp_and_zero() {
        assert_eq!(Settings::from_json(&json!({})).free_after_secs, 60);
        assert_eq!(Settings::from_json(&json!({"freeAfterSecs": 0})).free_after_secs, 0);
        assert_eq!(Settings::from_json(&json!({"freeAfterSecs": 99999})).free_after_secs, 3600);
        assert_eq!(Settings::from_json(&json!({"freeAfterSecs": -5})).free_after_secs, 60);
    }

    #[test]
    fn empty_shortcut_means_none_and_camel_case_roundtrip() {
        let s = Settings::from_json(&json!({"shortcut": "", "mouseGesture": false, "firefoxNoRemote": true}));
        assert_eq!(s.shortcut, "");
        assert!(!s.mouse_gesture && s.firefox_no_remote);
        let out = serde_json::to_value(&s).unwrap();
        assert!(out.get("holdMs").is_some() && out.get("mouseGesture").is_some());
    }
}
