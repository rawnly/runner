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
    path: PathBuf,

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
        "Watching {} for changes...",
        &args
            .path
            .to_str()
            .ok_or(anyhow!("unable to retrive path"))?
    );
    println!();

    run(file_type, &args).await?;

    for res in rx {
        let event = &res?;
        let kind = event.kind;

        if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = kind {
            print!("\x1B[2J\x1B[1;1H");
            println!("File changed...");
            println!();
            run(file_type, &args).await?;
        }
    }

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum FileType {
    Python,
    Node,
    Unsupported,
}

// tryfrom
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
            _ => Ok(Self::Unsupported),
        }
    }
}

async fn run(file_type: FileType, args: &Args) -> anyhow::Result<()> {
    let command = match args.command.clone() {
        Some(cmd) => cmd,
        None => match file_type {
            FileType::Node => "node",
            FileType::Python => "python3",
            _ => return Err(anyhow!("Unsupported file type")),
        }
        .to_string(),
    };

    Command::new(command)
        .arg(&args.path)
        .spawn()?
        .wait()
        .await?;

    Ok(())
}
