use crate::cli::Args;
use crate::config::Config;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Settings {
    pub pattern: String,
    pub path: PathBuf,
    pub ignore_case: bool,
    pub hidden: bool,
    pub follow: bool,
    pub no_ignore: bool,
    pub multi: bool,
    pub no_preview: bool,
    pub open: Option<String>,

    // preview/UI
    pub preview_width: String,
    pub preview_context: usize,
}

impl Settings {
    pub fn from(args: &Args, cfg: &Config) -> Self {
        let def_preview_width = "right:60%".to_string();
        let def_preview_context = 2usize;

        Self {
            pattern: args.pattern.clone(),
            path: args.path.clone(),
            ignore_case: args.ignore_case || cfg.ignore_case.unwrap_or(false),
            hidden: args.hidden || cfg.hidden.unwrap_or(false),
            follow: args.follow || cfg.follow.unwrap_or(false),
            no_ignore: args.no_ignore || cfg.no_ignore.unwrap_or(false),
            multi: args.multi || cfg.multi.unwrap_or(false),
            no_preview: args.no_preview || cfg.no_preview.unwrap_or(false),
            open: args.open.clone().or_else(|| cfg.open.clone()),
            preview_width: cfg.preview_width.clone().unwrap_or(def_preview_width),
            preview_context: cfg.preview_context.clone().unwrap_or(def_preview_context),
        }
    }
}
