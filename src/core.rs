use crate::{command::docker::DockerError, file_type::FileType};
use std::{path::PathBuf, time::Duration};

use crate::command::docker;
use anyhow::anyhow;
use tokio::process::Command;

pub const DURATION_ZERO: Duration = Duration::from_secs(0);

pub async fn run(
    file_type: &FileType,
    no_docker: bool,
    command: Option<String>,
    env: Option<Vec<String>>,
    path: &PathBuf,
) -> anyhow::Result<(Duration, Duration)> {
    let passed_env: Vec<(String, String)> = env
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

    if !no_docker {
        match file_type.get_docker_image() {
            Some(_) => {
                let run_start = std::time::Instant::now();

                docker::run(file_type, path.to_str().unwrap(), passed_env).await?;
                let run_elapsed = run_start.elapsed();
                return Ok((DURATION_ZERO, run_elapsed));
            }
            None => {
                println!("Running without docker... ");
                println!("Unsupported docker runtime: '{}'", file_type.to_string());
            }
        }
    }

    match command.clone() {
        Some(c) => {
            let run_start = std::time::Instant::now();

            if c.contains(' ') {
                let mut parts = c.split_whitespace();
                let mut command = Command::new(parts.next().unwrap());

                command
                    .envs(passed_env.iter().cloned())
                    .args(parts)
                    .arg(path)
                    .spawn()?
                    .wait()
                    .await?;
            } else {
                let mut command = Command::new(c);
                command
                    .envs(passed_env.iter().cloned())
                    .arg(path)
                    .spawn()?
                    .wait()
                    .await?;
            }

            Ok((DURATION_ZERO, run_start.elapsed()))
        }
        None => {
            if !file_type.is_available().await? {
                return Err(anyhow!(
                    "cannot find the required command: '{}'",
                    file_type.get_command_name().await?
                ));
            }

            if matches!(file_type, FileType::Rust) {
                let stem = path.file_stem().unwrap().to_str().unwrap();

                let build_start = std::time::Instant::now();
                Command::new(file_type.get_command_name().await?)
                    .envs(passed_env.iter().cloned())
                    .arg(path)
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
                FileType::Unsupported => {
                    return Err(anyhow!("Unsupported file type: '.{:?}'", path.extension()))
                }
                _ => Command::new(file_type.get_command_name().await?),
            };

            let arguments = match file_type {
                FileType::Go => vec!["run"],
                FileType::Typescript => vec!["run"],
                _ => vec![],
            };

            let run_start = std::time::Instant::now();
            command
                .envs(passed_env.iter().cloned())
                .args(arguments)
                .arg(path)
                .spawn()?
                .wait()
                .await?;

            Ok((DURATION_ZERO, run_start.elapsed()))
        }
    }
}
