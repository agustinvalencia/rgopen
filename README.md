# rgopen

A blazingly fast and total overkill CLI to grep and open files

> TLDR: I am a nerd and wrote a totally unnecessary shell tool to grep and open files that I am sure I will use quite a lot because I am lazy.

Sometimes you're working on your shell, and forgot which file had the thing you are looking for now, and then you start grepping and once you've found it you have to use your mouse (or terminal in vim/emacs mode) to copy the file name and launch whatever tool you would use to read that which is too much effort. Of course I could open my `nvim` and use telescope/fzflua/whatever and start searching there. But I want to do it right from the console because that's how I am.

The thing is I am really bad at recalling things, let alone, huge `rg` , `fzf` and `xargs` pipes with `awk` magic just to open the matches in nvim, is something I had tried several times and every time I have to go googling to remember how the f* * k the syntax was. 

I know I could ask GPT to write a `bash`/`zsh` function paste it in my .zshrc and forget about it. But why would I do that if I can overengineer stuff for dopamine shots, plus I really don't like bash scripts syntax.

I've been going through a writing CLI phase lately, so I will do so... in Rust, because it's trendy, and want to flex as chad programmer (and claim it is "blazingly fast"). So let's do it. I will start explaining stuff while building the simplest cli possible and then will continue building on top of it a more complex stuff so I really flex. 


# Launching sys processes from rust

Create your project using `cargo` and put this in your `Cargo.toml`

```toml
[package]
name = "rgopen"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1.0.100"
clap = {version="4.5.50", features=["derive"]}
which = "8.0.0"
```

Let's for now assume you've installed ripgrep `rg` and skim `sk` (the rust-based `fzf`). If you don't, go and get them with `cargo install ripgrep skim`

Just to have a taste of it, let's try this out first.
```rust
use std::process::Command 

fn main() -> Result<()> {
    let output = Command::new("rg")
        .args(["TODO"])
        .output()
        .context("failed to run ripgrep")?;
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
```

Here we use `Command` to make a system call that resembles writing `rg TODO` in the shell.

To test it just run 
```shell
cargo run
```

# Piping shell commads from Rust

Now, let's pipe grep's matches to `sk`: 

```rust
fn main() -> Result<()> {
    let rg = Command::new("rg")
        .args(["-l", "TODO"])
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to run ripgrep")?;

    let mut sk = Command::new("sk")
        .stdin(rg.stdout.unwrap())
        .spawn()
        .context("failed to start sk")?;

    let status = sk.wait().context("waiting for skim failed")?;
    if !status.success() {
        eprint!("skim exited with status {:?}", status.code());
    }
    Ok(())
}
```

Good. But now we're just getting the selection in the terminal due to sk writing to stdout, not our own command. So, let's capture it and print it ourselves 

```rust
fn main() -> Result<()> {
    let rg = Command::new("rg")
        .args(["-l", "TODO"])
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to run ripgrep")?;

    let output = Command::new("sk")
        .stdin(rg.stdout.unwrap())
        .output()
        .context("failed to run sk")?;

    if output.status.success() {
        let selection = String::from_utf8_lossy(&output.stdout);
        println!("Selected:\n->{selection}");
    }

    Ok(())
}
```

We're making good progress. Let's add some preview just leveraging `sk` itself.

```rust
fn main() -> Result<()> {
    let rg = Command::new("rg")
        .args(["-l", "TODO"])
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to run ripgrep")?;

    let preview_cmd = "rg --color=always --line-number 'TODO' {} | head -n 200";
    let output = Command::new("sk")
        .args([
            "--ansi",
            "--preview",
            preview_cmd,
            "--preview-window",
            "right:60%",
        ])
        .stdin(rg.stdout.unwrap())
        .output()
        .context("failed to run sk")?;

    if output.status.success() {
        let selection = String::from_utf8_lossy(&output.stdout);
        println!("Selected:\n->{selection}");
    }

    Ok(())
}
```

![[Pasted image 20251024172242.png]]

Ok, so now we have a working starting point. Let's then make this a CLI itself using `clap`. Let's put some order in the code as well, so it is scalable and maintainable (idk if I would maintain it anyhow).

# Cleaning the house.

Let's create `src/cli.rs` and use `clap` to define a cool CLI. For now, we will basically define the command arguments and let `clap` do all the heavy lifting for us. 

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "rgopen", version, about = "ripgrep + skim for lazy nerds")]
pub struct Args {
    /// Regex pattern to search for (passed to ripgrep)
    pub pattern: String,

    // Disable preview pane
    #[arg(long)]
    pub no_preview: bool,

    /// Directory to search from
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
}
```

Also, let's organise the commands generation a little in `src/pipeline.rs`: 

```rust
use crate::cli::Args;
use anyhow::{Context, Result};
use std::ffi::OsString;
use std::process::{Command, Stdio};
use which::which;

// Check if the tool dependencies are installed
pub fn check_binaries() -> Result<()> {
    for bin in ["rg", "sk"] {
        which(bin).with_context(|| format!("{bin} not found in PATH"))?;
    }
    Ok(())
}

// ripgrep command building
fn build_rg_args(args: &Args) -> Vec<OsString> {
    let mut v = vec!["-l".into(), "--color=never".into(), "--no-messages".into()];
    v.push(args.pattern.clone().into());
    v.push(args.path.clone().into_os_string());
    v
}

// Preview command for skim, use ripgrep to highlight matches 
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

// main entry point for the pipeline
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
```

Here, we use an inner scope to make sure the commands themselves have finished.


Then our `main.rs` looks cleaner as : 

```rust
mod cli;
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

    Ok(())
}
```


# Opening the file

Now, we've used ripgrep to find files with some content, then use skim to interactively pick the ones we want. Now to call my lazyness goal reached, I need to open the file(s). We won't do anything new, just keep piping and exposing a new command argument for a custom app to open the file. 

Let's modify the command definition: 

```rust
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

    // Open the selection
    #[arg(long)]
    pub open: Option<String>,
}
```


Let's create a new module `src/open.rs` in which we use the command received by `--open`
 
```rust
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
```

and the main entrypoint updated: 

```rust
mod cli;
mod open;
mod pipeline;

use anyhow::Result;

use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    pipeline::check_binaries()?;
    let picks = pipeline::run_pipeline(&args)?;

    if let Some(cmd) = args.open {
        open::open_with(&cmd, &picks)?;
    }

    Ok(())
}
```


# Now, the total overkill: A Config file

As I am extra lazy, I thought "what if I write a config file to store my own defaults so my commands would be even shorter". And if there are some other lazy nerds like me, they could set whatever they want as well (without having to recompile rust sources).

So, hold on that we are in the last part of my overengineered command sort of ripgrep ripoff. 

I want to allow users to have a config file either in `~/.rgopen/config.toml` or `~/.config/rgopen/config.toml`, whereas the latter precedes the former. Users could also use `config.yaml` or `config.yml` as a matter of taste freedom. 

Let's first write a struct to hold the configs and the code that look for it in your disk and fill the values accordingly in `src/config.rs`

```rust
use anyhow::{Context, Result};
use std::env::home_dir;
use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub no_preview: Option<bool>,
    pub open: Option<String>,
}

pub fn load_config() -> Result<Config> {
    let home = home_dir().context("Could not determine home directory")?;

    // preferred lookup order
    let config_dirs = vec![home.join(".config/rgopen"), home.join(".rgopen")];

    for dir in &config_dirs {
        for name in ["config.toml", "config.yaml", "config.yml"] {
            let path = dir.join(name);
            if path.exists() {
                let s = fs::read_to_string(&path)
                    .with_context(|| format!("reading {}", path.display()))?;
                return match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
                    "toml" => Ok(toml::from_str(&s)?),
                    "yaml" | "yml" => Ok(serde_yaml::from_str(&s)?),
                    _ => Ok(Config::default()),
                };
            }
        }
    }
    Ok(Config::default())
}
```

But we also need to have a way to merge whatever args we got from stdin with the defaults, of course allowing stdin to override the defaults in the config file. For such, let's create `src/settings.rs` to handle such and adapt `src/main.rs` accordingly as well. 

`settings.rs`
```rust
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
```


and the entry point in `main.rs`

```rust
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

    if let Some(cmd) = options.open {
        open::open_with(&cmd, &picks)?;
    }

    Ok(())
}
```

So now if I have this `~/.config/rgopen/config.toml`

```toml
open = "nvim"
```

I can drop the `--open nvim` piece when typing the command. Though, if I would like to use vscode for a particular case I can still put `--open code` and it will override my config default. 

Voilá, a saturday afternoon wasted in this totally unnecessary command that I am sure I will use a lot. 

Thanks to ripgrep and skim developers for such great tools!

I may keep adding stuff to this, so make sure to star the [repository here.](https://github.com/agustinvalencia/rgopen)

Or just install it by

```bash
cargo install --git https://github.com/agustinvalencia/rgopen
`
