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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file_type = FileType::try_from(&args.path)?;

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    watcher.watch(&args.path, RecursiveMode::Recursive)?;

    // clear screen
    print!("{}[2J", 27 as char);

    println!(
        "Watching {} for changes...",
        &args
            .path
            .to_str()
            .ok_or(anyhow!("unable to retrive path"))?
    );

    run(file_type, &args.path).await?;

    for res in rx {
        let event = &res?;
        let kind = event.kind;

        if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = kind {
            run(file_type, &args.path).await?;
        }
    }

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum FileType {
    Python,
    Node,
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
            _ => Err(anyhow!("Unsupported file type")),
        }
    }
}

async fn run(file_type: FileType, path: &PathBuf) -> anyhow::Result<()> {
    let command = match file_type {
        FileType::Node => "node",
        FileType::Python => "python3",
    };

    Command::new(command).arg(path).spawn()?.wait().await?;

    Ok(())
}
