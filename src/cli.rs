use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rgopen", version, about = "ripgrep + skim for lazy nerds")]
pub struct Args {
    /// Regex pattern to search for (passed to ripgrep)
    pub pattern: String,

    /// Directory to search from
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Case-insensitive search (-i)
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Disable preview pane
    #[arg(long)]
    pub no_preview: bool,
}
