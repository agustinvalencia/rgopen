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

    // Disable preview pane
    #[arg(long)]
    pub no_preview: bool,

    // Open the selection
    #[arg(long)]
    pub open: Option<String>,
}
