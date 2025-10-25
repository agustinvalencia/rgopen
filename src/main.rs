mod cli;
mod config;
mod open;
mod pipeline;
mod settings;

use anyhow::Result;

use clap::Parser;

fn main() -> Result<()> {
    pipeline::check_binaries()?;

    let args = cli::Args::parse();
    let cfg = config::load_config()?;
    let options = settings::Settings::from(&args, &cfg);
    let picks = pipeline::run_pipeline(&options)?;

    // for p in &picks {
    //     println!("{p}");
    // }

    if let Some(cmd) = options.open {
        open::open_with(&cmd, &picks)?;
    }

    Ok(())
}
