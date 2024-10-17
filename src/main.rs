mod cli;
mod command;
mod core;
mod file_type;
mod templates;
mod utils;

use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use crate::{
    cli::Args,
    core::{run, DURATION_ZERO},
    file_type::FileType,
    utils::{clear_screen, temp_file},
};

use anyhow::anyhow;
use clap::Parser;
use colored::*;
use notify::{
    event::{DataChange, ModifyKind},
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::{select, sync::mpsc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut last_run_duration = DURATION_ZERO;

    let runtime = args.runtime.clone().unwrap_or(FileType::Typescript);
    let path = args.path.clone().unwrap_or(temp_file(&runtime).await?);
    let is_temp = args.path.is_none();

    if args.path.is_some() && args.runtime.is_some() {
        println!("🚨 Both path and runtime are provided, ignoring runtime");
    }

    let file_type = FileType::try_from(&path)?;

    let (tx, mut rx) = mpsc::channel(10);
    let mut watcher = RecommendedWatcher::new(
        move |result: std::result::Result<Event, notify::Error>| {
            tx.blocking_send(result).expect("failed to send event");
        },
        notify::Config::default(),
    )?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    clear_screen();
    eprintln!(
        "🏃 Watching {} for changes...",
        &path
            .to_str()
            .ok_or(anyhow!("unable to retrive path"))?
            .yellow()
    );
    eprintln!();

    // tokio::spawn(async move {
    //     loop {
    //
    //         // on keyboard `Enter`, refresh
    //         for c in stdin.keys() {
    //             clear_screen();
    //
    //             match c.unwrap() {
    //                 Key::Esc => {
    //                     eprintln!("exc pressed");
    //                     break;
    //                 }
    //                 _ => break,
    //             }
    //         }
    //     }
    // });

    loop {
        select! {
            res = rx.recv() => {
                let event = res.ok_or(anyhow!("Failed to receive event"))??;
                let kind = event.kind;

                match kind {
                    EventKind::Create(_) | EventKind::Modify(ModifyKind::Data(DataChange::Content)) => {
                        clear_screen();

                        eprintln!("🏃File Changed!");
                        eprintln!();


                        let image = args.image.clone().map(|i| i.parse().unwrap());

                        let (
                            build_duration,
                            run_duration,
                            docker_image
                        ) = run(
                            &file_type,
                            args.no_docker,
                            args.command.clone(),
                            args.env.clone(),
                            &path,
                            image
                        ).await?;

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

                        if let Some(image) = docker_image {
                            eprintln!("📦 Image: {}", image);
                        }

                        eprintln!(
                            "🏃 Watching {} for changes...",
                            &path
                                .to_str()
                                .ok_or(anyhow!("unable to retrive path"))?
                                .yellow()
                        );

                        last_run_duration = elapsed;
                    }
                    _ => {}
                }
            }

            _ = tokio::signal::ctrl_c() => {
                println!("🧼 Cleaning up...");

                if is_temp {
                    tokio::fs::remove_file(&path).await?;
                }

                break;
            }
        }
    }

    Ok(())
}
