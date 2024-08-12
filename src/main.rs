use std::{env, path::PathBuf, sync::mpsc, time::Duration};

use anyhow::anyhow;
use clap::Parser;
use colored::*;
use notify::{
    event::{DataChange, ModifyKind},
    EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::process::Command;

#[derive(Debug, Clone, Parser)]
struct Args {
    /// path to the file to watch
    path: PathBuf,

    /// command to run when the file changes -
    /// if includes whitespace, it will be split and the first part will be the command
    command: Option<String>,

    #[clap(short)]
    env: Option<Vec<String>>,
}

fn clear_screen() {
    eprint!("\x1B[2J\x1B[1;1H");
}

const DURATION_ZERO: Duration = Duration::from_secs(0);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut last_run_duration = DURATION_ZERO;

    let file_type = FileType::try_from(&args.path)?;

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    watcher.watch(&args.path, RecursiveMode::Recursive)?;

    // clear screen
    clear_screen();

    eprintln!(
        "🏃 Watching {} for changes...",
        &args
            .path
            .to_str()
            .ok_or(anyhow!("unable to retrive path"))?
            .yellow()
    );
    eprintln!();

    let (build_duration, run_duration) = run(&file_type, &args).await?;

    let elapsed = run_duration + build_duration;
    let time_taken = format!("{:?}", elapsed).dimmed();

    eprintln!();
    eprintln!(
        "🏁 Run taken: {} [{:?} build, {:?} run]",
        time_taken, build_duration, run_duration
    );

    last_run_duration = elapsed;

    for res in rx {
        let event = &res?;
        let kind = event.kind;

        // measure the time taken to reload the file
        if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = kind {
            clear_screen();

            eprintln!("Lap! 🏃");
            eprintln!();

            let (build_duration, run_duration) = run(&file_type, &args).await?;

            let elapsed = run_duration + build_duration;
            let time_taken = format!("{:?}", elapsed).dimmed();

            eprintln!();
            eprintln!(
                "🏁 Run taken: {} [{:?} build, {:?} run]",
                time_taken, build_duration, run_duration
            );

            let delta = if last_run_duration.gt(&elapsed) {
                last_run_duration - elapsed
            } else {
                elapsed - last_run_duration
            };

            let deltastring = if last_run_duration.lt(&elapsed) && !delta.is_zero() {
                format!("+{:?}", delta).red()
            } else {
                format!("-{:?}", delta).green()
            };

            eprintln!("⏱️ Delta: {}", deltastring);

            last_run_duration = elapsed;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum FileType {
    Python,
    Python3,
    Node,
    Go,
    Bun,
    Rust,
    Shell,
    Unsupported(String),
}

impl TryFrom<&PathBuf> for FileType {
    type Error = anyhow::Error;

    fn try_from(value: &PathBuf) -> std::prelude::v1::Result<Self, Self::Error> {
        let ext = value
            .extension()
            .and_then(|f| f.to_str())
            .ok_or(anyhow!("cannot extract file extension"))?;

        match ext {
            "py" => Ok(Self::Python3),
            "sh" => Ok(Self::Shell),
            "js" | "mjs" => Ok(Self::Node),
            "go" => Ok(Self::Go),
            "ts" => Ok(Self::Bun),
            "rs" => Ok(Self::Rust),
            _ => Ok(Self::Unsupported(ext.to_string())),
        }
    }
}

impl ToString for FileType {
    fn to_string(&self) -> String {
        if matches!(self, Self::Shell) {
            if cfg!(windows) {
                return "bash".to_string();
            } else {
                return env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            }
        }

        match self {
            Self::Python => "python",
            Self::Python3 => "python3",
            Self::Node => "node",
            Self::Go => "go",
            Self::Bun => "bun",
            Self::Rust => "rustc",
            _ => "Unsupported",
        }
        .to_string()
    }
}

impl FileType {
    async fn is_available(&self) -> Result<bool, anyhow::Error> {
        match self {
            Self::Unsupported(ext) => Err(anyhow!("Unsupported file type: '.{}'", ext)),
            // if python3 is not available, we can use python
            Self::Python | Self::Python3 => Ok(Command::new(Self::Python3.to_string())
                .arg("--version")
                .output()
                .await
                .is_ok()
                || Command::new(Self::Python.to_string())
                    .arg("--version")
                    .output()
                    .await
                    .is_ok()),

            _ => Ok(Command::new(self.to_string())
                .arg("--version")
                .output()
                .await
                .is_ok()),
        }
    }

    async fn get_command_name(&self) -> Result<String, anyhow::Error> {
        match self {
            Self::Unsupported(ext) => Err(anyhow!("Unsupported file type: '.{}'", ext)),
            Self::Python3 => {
                if self.is_available().await.is_ok() {
                    return Ok("python3".to_string());
                }

                Ok(Self::Python.to_string())
            }
            _ => Ok(self.to_string()),
        }
    }
}

async fn run(file_type: &FileType, args: &Args) -> anyhow::Result<(Duration, Duration)> {
    let start = std::time::Instant::now();
    let passed_env: Vec<(String, String)> = args
        .env
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|e| {
            let mut parts = e.split('=');
            (
                parts.next().unwrap().to_string(),
                parts.next().unwrap().to_string(),
            )
        })
        .collect();

    match args.command.clone() {
        Some(c) => {
            let run_start = std::time::Instant::now();

            if c.contains(' ') {
                let mut parts = c.split_whitespace();
                let mut command = Command::new(parts.next().unwrap());

                command
                    .envs(passed_env.iter().cloned())
                    .args(parts)
                    .arg(&args.path)
                    .spawn()?
                    .wait()
                    .await?;
            } else {
                let mut command = Command::new(c);
                command
                    .envs(passed_env.iter().cloned())
                    .arg(&args.path)
                    .spawn()?
                    .wait()
                    .await?;
            }

            return Ok((DURATION_ZERO, run_start.elapsed()));
        }
        None => {
            if !file_type.is_available().await? {
                return Err(anyhow!(
                    "cannot find the required command: '{}'",
                    file_type.get_command_name().await?
                ));
            }

            if matches!(file_type, FileType::Rust) {
                let stem = args.path.file_stem().unwrap().to_str().unwrap();

                let build_start = std::time::Instant::now();
                Command::new(file_type.get_command_name().await?)
                    .envs(passed_env.iter().cloned())
                    .arg(&args.path)
                    .arg("-o")
                    .arg(&format!("/tmp/{}-runner-build", stem))
                    .spawn()?
                    .wait()
                    .await?;
                let build_elapsed = build_start.elapsed();

                let run_start = std::time::Instant::now();
                Command::new(&format!("/tmp/{}-runner-build", stem))
                    .envs(passed_env.iter().cloned())
                    .spawn()?
                    .wait()
                    .await?;
                let run_elapsed = run_start.elapsed();

                return Ok((build_elapsed, run_elapsed));
            }

            let mut command = match file_type {
                FileType::Unsupported(ext) => {
                    return Err(anyhow!("Unsupported file type: '.{}'", ext))
                }
                _ => Command::new(file_type.get_command_name().await?),
            };

            let arguments = match file_type {
                FileType::Go => vec!["run"],
                FileType::Bun => vec!["run"],
                _ => vec![],
            };

            let run_start = std::time::Instant::now();
            command
                .envs(passed_env.iter().cloned())
                .args(arguments)
                .arg(&args.path)
                .spawn()?
                .wait()
                .await?;

            Ok((DURATION_ZERO, run_start.elapsed()))
        }
    }
}
