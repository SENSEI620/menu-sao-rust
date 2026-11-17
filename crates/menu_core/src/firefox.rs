//! Importar os perfis do Firefox: os perfis novos (banco `Profile Groups/*.sqlite`,
//! com ícone e nome de exibição) e os antigos (`profiles.ini`).

use crate::model::Profile;
use crate::uid;
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::{Path, PathBuf};

/// %APPDATA%\Mozilla\Firefox
pub fn firefox_root() -> PathBuf {
    dirs::config_dir().unwrap_or_default().join("Mozilla").join("Firefox")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selectable {
    pub name: String,
    pub path: PathBuf,
    pub avatar: String,
}

const AVATARS: &[(&str, &str)] = &[
    ("game", "🎮"), ("leaf", "🍃"), ("flower", "✿"), ("book", "📖"), ("brief", "💼"),
    ("heart", "❤️"), ("shop", "🛍️"), ("cart", "🛒"), ("star", "⭐"), ("soccer", "⚽"),
    ("gem", "💎"), ("music", "🎵"), ("travel", "✈️"), ("paint", "🎨"), ("paw", "🐾"),
];

/// Converte o nome do avatar do Firefox em emoji (🦊 se não reconhecer).
pub fn avatar_icon(avatar: &str) -> &'static str {
    let a = avatar.to_lowercase();
    AVATARS.iter().find(|(k, _)| a.contains(k)).map(|(_, e)| *e).unwrap_or("🦊")
}

/// Lê os perfis novos. Copia o banco antes, porque o Firefox pode estar com ele aberto.
pub fn read_selectable(root: &Path, log: &dyn Fn(&str)) -> Vec<Selectable> {
    let dir = root.join("Profile Groups");
    let Ok(rd) = fs::read_dir(&dir) else {
        log("pasta Profile Groups não encontrada");
        return Vec::new();
    };
    let mut files: Vec<PathBuf> =
        rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "sqlite")).collect();
    files.sort();

    let mut out = Vec::new();
    for f in files {
        let rows = match read_profiles_table(&f) {
            Ok(r) => r,
            Err(e) => {
                log(&format!("leitura dos perfis novos falhou: {e}"));
                continue;
            }
        };
        for (name, p, avatar) in rows {
            let base = Path::new(&p).file_name().map(PathBuf::from).unwrap_or_default();
            let cands = [PathBuf::from(&p), root.join(&p), root.join("Profiles").join(&p), root.join("Profiles").join(base)];
            let full = cands.into_iter().find(|c| c.is_absolute() && c.exists());
            log(&format!(
                "perfil novo: {name} | {p} | {}",
                full.as_ref().map(|x| x.display().to_string()).unwrap_or_else(|| "PASTA NAO ENCONTRADA".into())
            ));
            if let Some(path) = full {
                out.push(Selectable { name, path, avatar });
            }
        }
    }
    out
}

fn read_profiles_table(db: &Path) -> Result<Vec<(String, String, String)>, String> {
    let tmp = std::env::temp_dir().join(format!("menu-sao-prof-{}-{}.sqlite", std::process::id(), uid()));
    fs::copy(db, &tmp).map_err(|e| e.to_string())?;
    let wal = PathBuf::from(format!("{}-wal", db.display()));
    if wal.exists() {
        let _ = fs::copy(&wal, PathBuf::from(format!("{}-wal", tmp.display())));
    }
    let result = (|| -> Result<Vec<(String, String, String)>, rusqlite::Error> {
        let conn = Connection::open_with_flags(&tmp, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let mut st = conn.prepare("SELECT name, path, avatar FROM Profiles")?;
        let rows = st.query_map([], |r| {
            Ok((
                r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ))
        })?;
        rows.collect()
    })();
    let _ = fs::remove_file(&tmp);
    let _ = fs::remove_file(format!("{}-wal", tmp.display()));
    let _ = fs::remove_file(format!("{}-shm", tmp.display()));
    result.map_err(|e| e.to_string())
}

/// Nomes dos perfis antigos do `profiles.ini` (seções [Profile0], [Profile1]...).
pub fn read_ini_names(root: &Path) -> Vec<String> {
    let Ok(txt) = fs::read_to_string(root.join("profiles.ini")) else { return Vec::new() };
    let mut names = Vec::new();
    let mut in_profile = false;
    for line in txt.lines() {
        let s = line.trim();
        if s.starts_with('[') {
            let inner = s.trim_start_matches('[').trim_end_matches(']').to_ascii_lowercase();
            in_profile = inner.strip_prefix("profile").is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
        } else if in_profile {
            if let Some(v) = s.strip_prefix("Name=") {
                if !v.is_empty() {
                    names.push(v.to_string());
                }
            }
        }
    }
    names
}

/// Junta os perfis do Firefox à lista. Devolve quantos foram adicionados ou corrigidos.
pub fn import(cfg: &mut Vec<Profile>, root: &Path, log: &dyn Fn(&str)) -> usize {
    let mut n = 0;
    let has = |cfg: &Vec<Profile>, pf: &str| cfg.iter().any(|x| x.br == "firefox" && x.pf == pf);

    for s in read_selectable(root, log) {
        let path = s.path.display().to_string();
        if has(cfg, &path) {
            continue;
        }
        // corrige entradas antigas que usavam só o nome
        if let Some(old) = cfg.iter_mut().find(|x| x.br == "firefox" && x.pf == s.name) {
            old.pf = path;
            n += 1;
            continue;
        }
        cfg.push(Profile {
            id: uid(),
            name: s.name,
            icon: avatar_icon(&s.avatar).into(),
            br: "firefox".into(),
            pf: path,
            ..Default::default()
        });
        n += 1;
    }
    for name in read_ini_names(root) {
        if has(cfg, &name) {
            continue;
        }
        cfg.push(Profile { id: uid(), name: name.clone(), icon: "🦊".into(), br: "firefox".into(), pf: name, ..Default::default() });
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_root() -> (tempfile::TempDir, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().to_path_buf();
        fs::create_dir_all(root.join("Profile Groups")).unwrap();
        fs::create_dir_all(root.join("Profiles").join("abc.F1")).unwrap();
        let conn = Connection::open(root.join("Profile Groups").join("grupo.sqlite")).unwrap();
        conn.execute("CREATE TABLE Profiles (id INTEGER PRIMARY KEY, path TEXT, name TEXT, avatar TEXT, themeId TEXT)", []).unwrap();
        conn.execute("INSERT INTO Profiles (path, name, avatar) VALUES ('Profiles/abc.F1', 'F1-JOURNAL-NEWS', 'gem')", []).unwrap();
        conn.execute("INSERT INTO Profiles (path, name, avatar) VALUES ('Profiles/sumiu', 'Fantasma', 'star')", []).unwrap();
        drop(conn);
        fs::write(
            root.join("profiles.ini"),
            "[Install123]\nDefault=Profiles/x\n\n[Profile0]\nName=antigo\nIsRelative=1\nPath=Profiles/x\n\n[General]\nStartWithLastProfile=1\n",
        )
        .unwrap();
        (d, root)
    }

    #[test]
    fn avatar_to_emoji() {
        assert_eq!(avatar_icon("GEM-2"), "💎");
        assert_eq!(avatar_icon("desconhecido"), "🦊");
    }

    #[test]
    fn reads_new_profiles_and_skips_missing_folders() {
        let (_d, root) = make_root();
        let list = read_selectable(&root, &|_| {});
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "F1-JOURNAL-NEWS");
        assert!(list[0].path.ends_with("abc.F1"));
    }

    #[test]
    fn reads_ini_names_only_from_profile_sections() {
        let (_d, root) = make_root();
        assert_eq!(read_ini_names(&root), ["antigo"]);
    }

    #[test]
    fn import_adds_fixes_and_is_idempotent() {
        let (_d, root) = make_root();
        // entrada antiga que só tinha o nome é corrigida para o caminho da pasta
        let mut cfg = vec![Profile { id: "o".into(), name: "x".into(), br: "firefox".into(), pf: "F1-JOURNAL-NEWS".into(), ..Default::default() }];
        let n = import(&mut cfg, &root, &|_| {});
        assert_eq!(n, 2); // 1 corrigido + 1 do .ini
        assert!(cfg[0].pf.ends_with("abc.F1"));
        assert!(cfg.iter().any(|p| p.pf == "antigo"));
        assert_eq!(import(&mut cfg, &root, &|_| {}), 0);
    }

    #[test]
    fn missing_firefox_is_not_an_error() {
        let d = tempfile::tempdir().unwrap();
        let mut cfg = Vec::new();
        assert_eq!(import(&mut cfg, d.path(), &|_| {}), 0);
    }
}
