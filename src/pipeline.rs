use crate::cli::Args;
use anyhow::{Context, Result};
use std::ffi::OsString;
use std::process::{Command, Stdio};
use which::which;

pub fn check_binaries() -> Result<()> {
    for bin in ["rg", "sk"] {
        which(bin).with_context(|| format!("{bin} not found in PATH"))?;
    }
    Ok(())
}

fn build_rg_args(args: &Args) -> Vec<OsString> {
    let mut v = vec!["-l".into(), "--color=never".into(), "--no-messages".into()];
    v.push(args.pattern.clone().into());
    v.push(args.path.clone().into_os_string());
    v
}

// Preview command for skim
fn preview_cmd(pattern: &str) -> String {
    format!(
        "rg --color=always --line-number '{}' {{}} | head -n 200",
        pattern.replace('\'', "'\"'\"'")
    )
}

// Build arguments for skim
fn build_sk_args(args: &Args) -> Vec<OsString> {
    let mut v = vec![
        "--ansi".into(),
        "--prompt".into(),
        "files> ".into(),
        "--reverse".into(),
        "--expect".into(),
        "enter".into(),
    ];
    if !args.no_preview {
        v.push("--preview".into());
        v.push(preview_cmd(&args.pattern).into());
        v.push("--preview-window".into());
        v.push("right:60%".into());
    }
    v
}

pub fn run_pipeline(args: &Args) -> Result<Vec<String>> {
    let mut rg = Command::new("rg")
        .args(build_rg_args(args))
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to launch ripgrep")?;

    let mut sk = Command::new("sk")
        .args(build_sk_args(args))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to launch ripgrep")?;

    // pipe rg.stdout -> sk.stdin
    {
        let mut rg_out = rg.stdout.take().context("rg stdout missing")?;
        let mut sk_in = sk.stdin.take().context("sk stdin missing")?;
        std::thread::spawn(move || {
            let _ = std::io::copy(&mut rg_out, &mut sk_in);
        });
    }

    let output = sk.wait_with_output().context("waiting for skim failed")?;
    let _ = rg.wait();

    if !output.status.success() {
        return Ok(vec![]);
    }

    // first line = key ("enter"), then selected paths
    let text = String::from_utf8_lossy(&output.stdout);
    let mut lines = text.lines();
    let _ = lines.next();
    Ok(lines
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}
