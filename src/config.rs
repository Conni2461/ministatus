use std::path::{Path, PathBuf};
use std::time::SystemTime;

const DEFAULT_SEPARATOR: &str = " | ";

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_blocks")]
    pub blocks: Vec<String>,
    #[serde(default = "default_separator")]
    pub separator: String,
}

fn default_blocks() -> Vec<String> {
    crate::block::ALL.iter().map(|v| (*v).to_owned()).collect()
}

fn default_separator() -> String {
    DEFAULT_SEPARATOR.to_owned()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blocks: default_blocks(),
            separator: default_separator(),
        }
    }
}

impl Config {
    pub fn path(home: &str) -> PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
        PathBuf::from(base).join("ministatus/config.toml")
    }

    pub fn load(path: &Path) -> Result<Self, anyhow::Error> {
        match std::fs::read_to_string(path) {
            Ok(v) => Ok(toml::from_str(&v)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }
}

pub fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

#[cfg(test)]
mod tests {
    use super::{Config, DEFAULT_SEPARATOR, default_blocks};

    #[test]
    fn empty_config_keeps_every_default() {
        let c: Config = toml::from_str("").unwrap();
        assert_eq!(c.blocks, default_blocks());
        assert_eq!(c.separator, DEFAULT_SEPARATOR);
    }

    #[test]
    fn block_list_replaces_the_default_and_keeps_its_order() {
        let c: Config = toml::from_str(r#"blocks = ["clock", "cpu"]"#).unwrap();
        assert_eq!(c.blocks, ["clock", "cpu"]);
        assert_eq!(c.separator, DEFAULT_SEPARATOR);
    }

    #[test]
    fn separator_alone_leaves_the_block_list_intact() {
        let c: Config = toml::from_str(r#"separator = " · ""#).unwrap();
        assert_eq!(c.blocks, default_blocks());
        assert_eq!(c.separator, " · ");
    }

    #[test]
    fn unknown_block_names_survive_parsing() {
        let c: Config = toml::from_str(r#"blocks = ["cpuu"]"#).unwrap();
        assert_eq!(c.blocks, ["cpuu"]);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(toml::from_str::<Config>("blocsk = []").is_err());
    }

    #[test]
    fn defaults_cover_every_known_block() {
        assert_eq!(default_blocks(), crate::block::ALL);
    }
}
