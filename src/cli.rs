use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rgopen", version, about = "ripgrep + skim for lazy nerds")]
pub struct Args {
    // Regex pattern to search for (passed to ripgrep)
    pub pattern: String,

    // Directory to search from
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    // Case insensitive search (-i)
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    // Include hidden files (--hidden)
    #[arg(long)]
    pub hidden: bool,

    // Follow symlinks (-L)
    #[arg(short = 'L', long)]
    pub follow: bool,

    // Disable ignored files (.gitignore, etc)
    #[arg(long)]
    pub no_ignore: bool,

    // Allow multi-select in skim
    #[arg(long)]
    pub multi: bool,

    // Disable preview pane
    #[arg(long)]
    pub no_preview: bool,

    // Open the selection
    #[arg(long)]
    pub open: Option<String>,
}
