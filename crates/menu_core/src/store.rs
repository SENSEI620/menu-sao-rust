//! Onde os dados ficam: %APPDATA%\menu-sao\ (a mesma pasta da versão Electron,
//! então seus perfis, links, notas e cronômetros continuam valendo).
//!
//! Melhorias em relação à versão antiga:
//! - gravação segura (escreve num arquivo temporário e troca), sem risco de
//!   ficar com o arquivo pela metade se o PC desligar na hora;
//! - uma cópia de segurança (`config.bak.json`) a cada gravação;
//! - se o config.json estiver corrompido, ele é guardado ao lado em vez de
//!   ser apagado.

use crate::model::{parse_config, Profile};
use crate::settings::Settings;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Store {
    dir: PathBuf,
}

/// Resultado de ler o config.json.
#[derive(Debug, PartialEq)]
pub enum Loaded {
    /// não existe ainda (primeira execução)
    Missing,
    Ok(Vec<Profile>),
    /// estava corrompido; foi guardado em `saved_as` e a lista começa vazia
    Corrupt { saved_as: PathBuf },
}

impl Store {
    pub fn new(dir: PathBuf) -> Store {
        Store { dir }
    }

    pub fn default_dir() -> PathBuf {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("menu-sao")
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn config_path(&self) -> PathBuf {
        self.dir.join("config.json")
    }
    pub fn settings_path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }
    pub fn log_path(&self) -> PathBuf {
        self.dir.join("log.txt")
    }

    pub fn load_config(&self) -> Loaded {
        let path = self.config_path();
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Loaded::Missing,
            Err(_) => return self.quarantine(&path),
        };
        match parse_config(text.trim_start_matches('\u{feff}')) {
            Ok(cfg) => Loaded::Ok(cfg),
            Err(_) => self.quarantine(&path),
        }
    }

    fn quarantine(&self, path: &Path) -> Loaded {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let saved_as = self.dir.join(format!("config.corrompido-{ts}.json"));
        let _ = fs::rename(path, &saved_as);
        Loaded::Corrupt { saved_as }
    }

    /// Lista atual, ou vazia se não existir/estiver corrompida.
    pub fn config_or_empty(&self) -> Vec<Profile> {
        match self.load_config() {
            Loaded::Ok(c) => c,
            _ => Vec::new(),
        }
    }

    pub fn save_config(&self, cfg: &[Profile]) -> io::Result<()> {
        let path = self.config_path();
        if path.exists() {
            let _ = fs::copy(&path, self.dir.join("config.bak.json"));
        }
        let json = serde_json::to_string_pretty(cfg).map_err(io::Error::other)?;
        write_atomic(&path, json.as_bytes())
    }

    pub fn load_settings(&self) -> Settings {
        fs::read_to_string(self.settings_path())
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(t.trim_start_matches('\u{feff}')).ok())
            .map(|v| Settings::from_json(&v))
            .unwrap_or_default()
    }

    pub fn save_settings(&self, s: &Settings) -> io::Result<()> {
        let json = serde_json::to_string_pretty(s).map_err(io::Error::other)?;
        write_atomic(&self.settings_path(), json.as_bytes())
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    // No Windows, rename sobre um arquivo existente funciona (substitui).
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let d = tempfile::tempdir().unwrap();
        let s = Store::new(d.path().join("menu-sao"));
        (d, s)
    }

    #[test]
    fn missing_then_saved_then_loaded_with_backup() {
        let (_d, s) = store();
        assert_eq!(s.load_config(), Loaded::Missing);
        let p = Profile { id: "1".into(), name: "A".into(), ..Default::default() };
        s.save_config(&[p.clone()]).unwrap();
        assert_eq!(s.load_config(), Loaded::Ok(vec![p.clone()]));
        let q = Profile { id: "2".into(), name: "B".into(), ..Default::default() };
        s.save_config(&[q]).unwrap();
        // a cópia de segurança guarda a versão anterior
        let bak = fs::read_to_string(s.dir().join("config.bak.json")).unwrap();
        assert!(bak.contains("\"A\""));
    }

    #[test]
    fn corrupt_file_is_kept_not_erased() {
        let (_d, s) = store();
        fs::create_dir_all(s.dir()).unwrap();
        fs::write(s.config_path(), "{ quebrado").unwrap();
        match s.load_config() {
            Loaded::Corrupt { saved_as } => assert!(saved_as.exists()),
            other => panic!("esperava Corrupt, veio {other:?}"),
        }
        assert_eq!(s.load_config(), Loaded::Missing);
    }

    #[test]
    fn reads_file_with_bom() {
        let (_d, s) = store();
        fs::create_dir_all(s.dir()).unwrap();
        fs::write(s.config_path(), "\u{feff}[{\"name\":\"X\"}]").unwrap();
        assert!(matches!(s.load_config(), Loaded::Ok(c) if c[0].name == "X"));
    }

    #[test]
    fn settings_roundtrip() {
        let (_d, s) = store();
        assert_eq!(s.load_settings(), Settings::default());
        let mut c = Settings::default();
        c.volume = 40;
        c.shortcut = "Control+Shift+K".into();
        s.save_settings(&c).unwrap();
        assert_eq!(s.load_settings(), c);
    }
}
