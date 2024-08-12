use std::{path::PathBuf, sync::mpsc};

use anyhow::anyhow;
use clap::Parser;
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file_type = FileType::try_from(&args.path)?;

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    watcher.watch(&args.path, RecursiveMode::Recursive)?;

    // clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!(
        "🏃 Watching {} for changes...",
        &args
            .path
            .to_str()
            .ok_or(anyhow!("unable to retrive path"))?
    );
    println!();

    run(&file_type, &args).await?;

    for res in rx {
        let event = &res?;
        let kind = event.kind;

        if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = kind {
            print!("\x1B[2J\x1B[1;1H");
            println!("File changed...");
            println!();
            run(&file_type, &args).await?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum FileType {
    Python,
    Node,
    Go,
    Bun,
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
            "py" => Ok(Self::Python),
            "js" | "mjs" => Ok(Self::Node),
            "go" => Ok(Self::Go),
            "ts" => Ok(Self::Bun),
            _ => Ok(Self::Unsupported(ext.to_string())),
        }
    }
}

impl ToString for FileType {
    fn to_string(&self) -> String {
        match self {
            Self::Python => "python3",
            Self::Node => "node",
            Self::Go => "go",
            Self::Bun => "bun",
            Self::Unsupported(_) => "Unsupported",
        }
        .to_string()
    }
}

impl FileType {
    async fn is_available(&self) -> Result<bool, anyhow::Error> {
        match self {
            Self::Unsupported(ext) => return Err(anyhow!("Unsupported file type: '.{}'", ext)),
            _ => Ok(Command::new(self.to_string())
                .arg("--version")
                .output()
                .await
                .is_ok()),
        }
    }
}

async fn run(file_type: &FileType, args: &Args) -> anyhow::Result<()> {
    match args.command.clone() {
        Some(c) => {
            if c.contains(' ') {
                let mut parts = c.split_whitespace();
                let mut command = Command::new(parts.next().unwrap());

                command.args(parts).arg(&args.path).spawn()?.wait().await?;
            } else {
                let mut command = Command::new(c);
                command.arg(&args.path).spawn()?.wait().await?;
            }
        }
        None => {
            if !file_type.is_available().await? {
                return Err(anyhow!(
                    "cannot find the required command: '{}'",
                    file_type.to_string()
                ));
            }

            let mut command = match file_type {
                FileType::Unsupported(ext) => {
                    return Err(anyhow!("Unsupported file type: '.{}'", ext))
                }
                _ => Command::new(file_type.to_string()),
            };

            let arguments = match file_type {
                FileType::Go => vec!["run"],
                FileType::Bun => vec!["run"],
                _ => vec![],
            };

            command
                .args(arguments)
                .arg(&args.path)
                .spawn()?
                .wait()
                .await?;
        }
    }

    Ok(())
}
