use anyhow::{Context, Result};
use std::env::home_dir;
use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub ignore_case: Option<bool>,
    pub hidden: Option<bool>,
    pub follow: Option<bool>,
    pub no_ignore: Option<bool>,
    pub multi: Option<bool>,
    pub no_preview: Option<bool>,
    pub open: Option<String>,
    pub preview_width: Option<String>,
    pub preview_context: Option<usize>,
}

pub fn load_config() -> Result<Config> {
    let home = home_dir().context("Could not determine home directory")?;

    // preferred lookup order
    let config_dirs = vec![home.join(".config/rgopen"), home.join(".rgopen")];

    for dir in &config_dirs {
        for name in ["config.toml", "config.yaml", "config.yml"] {
            let path = dir.join(name);
            if path.exists() {
                let s = fs::read_to_string(&path)
                    .with_context(|| format!("reading {}", path.display()))?;
                return match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
                    "toml" => Ok(toml::from_str(&s)?),
                    "yaml" | "yml" => Ok(serde_yaml::from_str(&s)?),
                    _ => Ok(Config::default()),
                };
            }
        }
    }
    Ok(Config::default())
}
