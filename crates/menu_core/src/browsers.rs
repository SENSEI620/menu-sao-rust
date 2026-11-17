//! Achar o Firefox/Brave e abrir um link no perfil certo.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    Firefox,
    Brave,
}

impl Browser {
    pub fn parse(s: &str) -> Browser {
        if s.eq_ignore_ascii_case("brave") {
            Browser::Brave
        } else {
            Browser::Firefox
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Browser::Firefox => "Firefox",
            Browser::Brave => "Brave",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LaunchError {
    InvalidUrl,
    InvalidProfile,
    ProfileDirMissing(String),
    BrowserNotFound(Browser),
    Spawn(String),
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaunchError::InvalidUrl => write!(f, "Link inválido: use http:// ou https://"),
            LaunchError::InvalidProfile => write!(f, "Nome de perfil inválido."),
            LaunchError::ProfileDirMissing(p) => write!(f, "Pasta do perfil não encontrada: {p}"),
            LaunchError::BrowserNotFound(b) => write!(f, "{} não encontrado.", b.name()),
            LaunchError::Spawn(_) => write!(f, "Não consegui abrir o navegador."),
        }
    }
}

/// Lugares onde o executável costuma estar (Arquivos de Programas, x86 e pasta local).
pub fn exe_candidates(b: Browser, env: &dyn Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    let pf = env("ProgramFiles").unwrap_or_else(|| r"C:\Program Files".into());
    let pf86 = env("ProgramFiles(x86)").unwrap_or_else(|| r"C:\Program Files (x86)".into());
    let la = env("LOCALAPPDATA").unwrap_or_default();
    let tail = match b {
        Browser::Brave => r"\BraveSoftware\Brave-Browser\Application\brave.exe",
        Browser::Firefox => r"\Mozilla Firefox\firefox.exe",
    };
    [pf, pf86, la].into_iter().map(|base| PathBuf::from(format!("{base}{tail}"))).collect()
}

pub fn find_exe(b: Browser) -> Option<PathBuf> {
    exe_candidates(b, &|k| std::env::var(k).ok()).into_iter().find(|p| p.exists())
}

/// "C:\..." ou "\\servidor\..." = caminho de pasta; senão é só o nome do perfil.
fn looks_like_path(s: &str) -> bool {
    let b = s.as_bytes();
    (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/')) || s.starts_with(r"\\")
}

/// Monta os argumentos do navegador. Não executa nada (por isso dá para testar).
pub fn build_args(
    b: Browser,
    pf: &str,
    url: &str,
    no_remote: bool,
    dir_exists: &dyn Fn(&str) -> bool,
) -> Result<Vec<String>, LaunchError> {
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(LaunchError::InvalidUrl);
    }
    if pf.is_empty() || pf.starts_with('-') {
        return Err(LaunchError::InvalidProfile);
    }
    let is_path = looks_like_path(pf);
    if is_path && !dir_exists(pf) {
        return Err(LaunchError::ProfileDirMissing(pf.to_string()));
    }
    Ok(match b {
        Browser::Brave => vec![format!("--profile-directory={pf}"), url.to_string()],
        Browser::Firefox => {
            let mut a = vec![if is_path { "-profile" } else { "-P" }.to_string(), pf.to_string()];
            if no_remote {
                a.push("-no-remote".into());
            }
            a.push(url.to_string());
            a
        }
    })
}

/// Abre o link no navegador/perfil indicado. Devolve o que foi executado (para o log).
pub fn launch(b: Browser, pf: &str, url: &str, no_remote: bool) -> Result<(PathBuf, Vec<String>), LaunchError> {
    let args = build_args(b, pf, url, no_remote, &|p| Path::new(p).exists())?;
    let exe = find_exe(b).ok_or(LaunchError::BrowserNotFound(b))?;
    let mut cmd = Command::new(&exe);
    cmd.args(&args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP: o navegador não morre junto com o menu
        cmd.creation_flags(0x0000_0008 | 0x0000_0200);
    }
    cmd.spawn().map_err(|e| LaunchError::Spawn(e.to_string()))?;
    Ok((exe, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yes(_: &str) -> bool {
        true
    }
    fn no(_: &str) -> bool {
        false
    }

    #[test]
    fn only_http_and_https() {
        for bad in ["file:///c:/x", "javascript:alert(1)", "ftp://a", "claude.ai", ""] {
            assert_eq!(build_args(Browser::Brave, "Default", bad, false, &yes), Err(LaunchError::InvalidUrl));
        }
        assert!(build_args(Browser::Brave, "Default", "HTTPS://claude.ai", false, &yes).is_ok());
    }

    #[test]
    fn rejects_option_injection_in_profile() {
        assert_eq!(build_args(Browser::Firefox, "-no-remote", "https://a.b", false, &yes), Err(LaunchError::InvalidProfile));
        assert_eq!(build_args(Browser::Firefox, "", "https://a.b", false, &yes), Err(LaunchError::InvalidProfile));
    }

    #[test]
    fn brave_args() {
        let a = build_args(Browser::Brave, "Profile 2", "https://a.b", false, &yes).unwrap();
        assert_eq!(a, ["--profile-directory=Profile 2", "https://a.b"]);
    }

    #[test]
    fn firefox_by_name_and_by_folder() {
        let a = build_args(Browser::Firefox, "trabalho", "https://a.b", true, &yes).unwrap();
        assert_eq!(a, ["-P", "trabalho", "-no-remote", "https://a.b"]);
        let a = build_args(Browser::Firefox, r"C:\Users\x\Profiles\abc.default", "https://a.b", false, &yes).unwrap();
        assert_eq!(a, ["-profile", r"C:\Users\x\Profiles\abc.default", "https://a.b"]);
        assert_eq!(
            build_args(Browser::Firefox, r"C:\sumiu", "https://a.b", false, &no),
            Err(LaunchError::ProfileDirMissing(r"C:\sumiu".into()))
        );
    }

    #[test]
    fn candidates_use_the_windows_folders() {
        let env = |k: &str| match k {
            "ProgramFiles" => Some(r"D:\Prog".to_string()),
            "LOCALAPPDATA" => Some(r"C:\Users\x\AppData\Local".to_string()),
            _ => None,
        };
        let c = exe_candidates(Browser::Brave, &env);
        assert_eq!(c[0], PathBuf::from(r"D:\Prog\BraveSoftware\Brave-Browser\Application\brave.exe"));
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn error_messages_are_in_portuguese() {
        assert_eq!(LaunchError::BrowserNotFound(Browser::Firefox).to_string(), "Firefox não encontrado.");
    }
}
