//! Perfis e links. O formato é o mesmo do `config.json` da versão Electron, então
//! quem já usa o programa antigo mantém tudo. Campos desconhecidos (que você
//! adicionar no futuro na interface) são preservados em `extra`.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

fn lenient_string<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::String(s) => s,
        Value::Null => String::new(),
        other => other.to_string(),
    })
}

fn lenient_i64<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)).unwrap_or(0),
        _ => 0,
    })
}

fn lenient_blocks<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Block>, D::Error> {
    Ok(match Value::deserialize(d)? {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|v| serde_json::from_value::<Block>(v).ok())
            .collect(),
        _ => Vec::new(),
    })
}

/// Um link dentro de um perfil.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Block {
    #[serde(default, deserialize_with = "lenient_string")]
    pub id: String,
    /// título
    #[serde(default, deserialize_with = "lenient_string")]
    pub t: String,
    #[serde(default, deserialize_with = "lenient_string")]
    pub url: String,
    /// quando o limite volta (milissegundos desde 1970); 0 = liberado
    #[serde(default, deserialize_with = "lenient_i64")]
    pub until: i64,
    #[serde(default, deserialize_with = "lenient_string")]
    pub note: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Um perfil de navegador (a esfera da primeira coluna).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Profile {
    #[serde(default, deserialize_with = "lenient_string")]
    pub id: String,
    #[serde(default, deserialize_with = "lenient_string")]
    pub name: String,
    #[serde(default, deserialize_with = "lenient_string")]
    pub icon: String,
    /// "firefox" ou "brave"
    #[serde(default, deserialize_with = "lenient_string")]
    pub br: String,
    /// nome do perfil ou caminho da pasta
    #[serde(default, deserialize_with = "lenient_string")]
    pub pf: String,
    #[serde(default, deserialize_with = "lenient_blocks")]
    pub blocks: Vec<Block>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Lê a lista inteira de um JSON; qualquer coisa que não seja lista vira erro.
pub fn parse_config(text: &str) -> Result<Vec<Profile>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    match v {
        Value::Array(items) => Ok(items
            .into_iter()
            .filter_map(|i| serde_json::from_value::<Profile>(i).ok())
            .collect()),
        _ => Err("o arquivo não é uma lista de perfis".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_electron_format_with_float_until() {
        let json = r#"[{"id":"a1","name":"Brave","icon":"🦁","br":"brave","pf":"Default",
            "blocks":[{"id":"b1","t":"Claude","url":"https://claude.ai","until":1760000000123.5,"note":"x"}]}]"#;
        let cfg = parse_config(json).unwrap();
        assert_eq!(cfg[0].blocks[0].until, 1760000000123);
        assert_eq!(cfg[0].blocks[0].note, "x");
    }

    #[test]
    fn keeps_unknown_fields_for_future_features() {
        let json = r##"[{"id":"a","name":"N","icon":"i","br":"firefox","pf":"p","cor":"#fff",
            "blocks":[{"id":"b","t":"T","url":"u","until":0,"note":"","favorito":true}]}]"##;
        let cfg = parse_config(json).unwrap();
        let back = serde_json::to_value(&cfg).unwrap();
        assert_eq!(back[0]["cor"], "#fff");
        assert_eq!(back[0]["blocks"][0]["favorito"], true);
    }

    #[test]
    fn tolerates_nulls_and_missing_fields() {
        let json = r#"[{"id":null,"name":"X","blocks":null},{"name":"Y","blocks":[{"t":null,"until":null}]}]"#;
        let cfg = parse_config(json).unwrap();
        assert_eq!(cfg.len(), 2);
        assert_eq!(cfg[0].id, "");
        assert!(cfg[0].blocks.is_empty());
        assert_eq!(cfg[1].blocks[0].until, 0);
    }

    #[test]
    fn rejects_non_lists() {
        assert!(parse_config("{}").is_err());
        assert!(parse_config("isso não é json").is_err());
    }
}
