mod cli;
mod open;
mod pipeline;

use anyhow::Result;

use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    pipeline::check_binaries()?;
    let picks = pipeline::run_pipeline(&args)?;

    for p in &picks {
        println!("{p}");
    }

    if let Some(cmd) = args.open {
        open::open_with(&cmd, &picks)?;
    }

    Ok(())
}
