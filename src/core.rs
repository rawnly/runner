use crate::{
    command::docker::{DockerError, DockerImage},
    file_type::FileType,
};
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
    docker_image: Option<DockerImage>,
) -> anyhow::Result<(Duration, Duration, Option<String>)> {
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

                match docker::run(
                    file_type,
                    path.to_str().unwrap(),
                    command.clone(),
                    docker_image.clone(),
                )
                .await
                {
                    Ok((_, image)) => {
                        let run_elapsed = run_start.elapsed();
                        return Ok((DURATION_ZERO, run_elapsed, Some(image)));
                    }
                    Err(DockerError::ImageNotInstalled(image)) => {
                        eprintln!("Image not installed: '{}'", image);

                        if inquire::prompt_confirmation("Would you like to install it?")? {
                            let mut s = spinners::Spinner::new(
                                spinners::Spinners::Dots,
                                "Pulling image...".into(),
                            );
                            let result = docker::pull(&image).await?;

                            if result.status.success() {
                                s.stop_with_symbol("✔");

                                let run_start = std::time::Instant::now();
                                let (_, image) = docker::run(
                                    file_type,
                                    path.to_str().unwrap(),
                                    command.clone(),
                                    docker_image,
                                )
                                .await?;
                                let run_elapsed = run_start.elapsed();
                                return Ok((DURATION_ZERO, run_elapsed, Some(image)));
                            } else {
                                s.stop_with_symbol("✖");
                                return Err(anyhow::anyhow!(
                                    "Failed to pull image: '{}'\n {}",
                                    image,
                                    String::from_utf8(result.stderr).unwrap()
                                ));
                            }
                        }
                    }
                    Err(e) => return Err(anyhow::anyhow!(e)),
                }
            }
            None => {
                eprintln!("Running without docker... ");
                eprintln!("Unsupported docker runtime: '{}'", file_type.to_string());
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

            Ok((DURATION_ZERO, run_start.elapsed(), None))
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

                return Ok((build_elapsed, run_elapsed, None));
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

            Ok((DURATION_ZERO, run_start.elapsed(), None))
        }
    }
}
