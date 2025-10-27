use crate::settings::Settings;
use anyhow::{Context, Result};
use std::ffi::OsString;
use std::process::{Command, Stdio};
use which::which;

pub fn check_binaries() -> Result<()> {
    for bin in ["rg", "sk"] {
        which(bin).with_context(|| {
            format!("{bin} not found in PATH. Try installing it with cargo install {bin}")
        })?;
    }
    Ok(())
}

fn build_rg_args(args: &Settings) -> Vec<OsString> {
    let mut v = vec!["-l".into(), "--color=never".into(), "--no-messages".into()];
    if args.ignore_case {
        v.push("-i".into())
    };
    if args.hidden {
        v.push("--hidden".into())
    };
    if args.follow {
        v.push("-L".into())
    };
    if args.no_ignore {
        v.push("-uu".into())
    };
    v.push("--".into());

    v.push(args.pattern.clone().into());
    v.push(args.path.clone().into_os_string());
    v
}

// Preview command for skim
fn preview_cmd(s: &Settings) -> String {
    let prev_context = s.preview_context.to_string();
    let mut flags = vec![
        "--color=always",
        "--line-number",
        "--max-columns=300",
        "--no-messages",
        "--context",
        &prev_context,
    ];
    if s.ignore_case {
        flags.push("-i");
    }
    if s.hidden {
        flags.push("--hidden");
    }
    if s.follow {
        flags.push("-L");
    }
    if s.no_ignore {
        flags.push("-uu");
    }

    if cfg!(windows) {
        format!(
            "powershell -NoProfile -Command rg {flags} -- '{pat}' '{{}}' | Select-Object -First 200",
            flags = flags.join(" "),
            pat = s.pattern.replace('"', "`\"")
        )
    } else {
        let pat_posix = s.pattern.replace('\'', "'\"'\"'");
        format!(
            "sh -c \"rg {flags} -- '{pat}' '{{}}' | head -n 200\"",
            flags = flags.join(" "),
            pat = pat_posix
        )
    }
}

// Build arguments for skim
fn build_sk_args(s: &Settings) -> Vec<OsString> {
    let mut v = vec![
        "--ansi".into(),
        "--prompt".into(),
        "files> ".into(),
        "--reverse".into(),
        "--expect".into(),
        "enter".into(),
    ];
    if s.multi {
        v.push("--multi".into());
    }
    if !s.no_preview {
        v.push("--preview".into());
        v.push(preview_cmd(&s).into());
        v.push("--preview-window".into());
        v.push(s.preview_width.clone().into());
    }
    v
}

pub fn run_pipeline(s: &Settings) -> Result<Vec<String>> {
    let mut rg = Command::new("rg")
        .args(build_rg_args(s))
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to launch ripgrep")?;

    let mut sk = Command::new("sk")
        .args(build_sk_args(s))
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
