//! Log de diagnóstico simples (log.txt), que a bandeja abre em "Ver log".

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Logger {
    path: PathBuf,
}

impl Logger {
    /// Cria o logger e esvazia o arquivo se ele passou de 200 KB.
    pub fn new(path: PathBuf) -> Logger {
        if let Ok(m) = fs::metadata(&path) {
            if m.len() > 200_000 {
                let _ = fs::write(&path, "");
            }
        }
        Logger { path }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn log(&self, msg: impl AsRef<str>) {
        let line = format!("{} {}", clock(), msg.as_ref());
        eprintln!("{line}");
        if let Some(dir) = self.path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&self.path) {
            let _ = writeln!(f, "{line}");
        }
    }
}

/// Hora UTC no formato HH:MM:SS (igual ao log da versão antiga).
fn clock() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) % 86_400;
    format!("{:02}:{:02}:{:02}", secs / 3600, secs % 3600 / 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_lines_and_truncates_big_files() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("log.txt");
        let l = Logger::new(p.clone());
        l.log("olá");
        assert!(fs::read_to_string(&p).unwrap().contains("olá"));
        fs::write(&p, "x".repeat(300_000)).unwrap();
        let _ = Logger::new(p.clone());
        assert_eq!(fs::metadata(&p).unwrap().len(), 0);
    }
}
