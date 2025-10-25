use crate::cli::Args;
use crate::config::Config;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Settings {
    pub pattern: String,
    pub path: PathBuf,
    pub no_preview: bool,
    pub open: Option<String>,
}

impl Settings {
    pub fn from(args: &Args, cfg: &Config) -> Self {
        Self {
            pattern: args.pattern.clone(),
            path: args.path.clone(),
            no_preview: args.no_preview || cfg.no_preview.unwrap_or(false),
            open: args.open.clone().or_else(|| cfg.open.clone()),
        }
    }
}
