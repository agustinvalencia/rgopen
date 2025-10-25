use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn open_with(cmd: &str, files: &[impl AsRef<Path>]) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    let mut c = Command::new(cmd);
    for f in files {
        c.arg(f.as_ref());
    }
    let status = c
        .status()
        .with_context(|| format!("failed running opener: {cmd}"))?;
    if !status.success() {
        eprintln!("opener exited with status: {status}");
    }
    Ok(())
}
