use std::process::ExitStatus;

use crate::{file_type::FileType, spawn_command};

#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("Unsupported docker runtime")]
    UnsupportedRuntime(String),
}

pub struct DockerImage {
    image: String,
    tag: Option<String>,
}

impl DockerImage {
    pub fn new(image: &str, tag: &str) -> Self {
        Self {
            image: image.to_string(),
            tag: Some(tag.to_string()),
        }
    }

    pub fn get_image(&self) -> String {
        format!(
            "{}:{}",
            self.image,
            self.tag.clone().unwrap_or("latest".to_string())
        )
    }

    pub fn alpine(image: &str) -> Self {
        Self::new(image, "alpine")
    }

    pub fn latest(image: &str) -> Self {
        Self::new(image, "latest")
    }
}

impl ToString for DockerImage {
    fn to_string(&self) -> String {
        self.get_image()
    }
}

pub async fn run(
    ft: &FileType,
    filepath: &str,
    _: Vec<(String, String)>,
) -> anyhow::Result<ExitStatus> {
    let image = ft
        .get_docker_image()
        .ok_or(DockerError::UnsupportedRuntime(ft.to_string()))?
        .to_string();

    let entrypoint = ft
        .get_docker_entrypoint()
        .ok_or(DockerError::UnsupportedRuntime(ft.to_string()))?;

    let volume = format!("{filepath}:/root/app/{entrypoint}");
    let command = ft
        .get_docker_command()
        .ok_or(DockerError::UnsupportedRuntime(ft.to_string()))?;

    let exit_status = spawn_command!("docker", "run", "-v", &volume, "-it", &image, &command)?
        .wait()
        .await?;

    Ok(exit_status)

    // match ft {
    //     FileType::Node => node(filepath).await,
    //     FileType::Rust => rust(filepath).await,
    //     FileType::Python => python(filepath).await,
    //     FileType::Ruby => ruby(filepath).await,
    //     FileType::Go => go(filepath).await,
    //     FileType::Php => php(filepath).await,
    //     FileType::Perl => perl(filepath).await,
    //     FileType::Shell => bash(filepath).await,
    //     FileType::C => c(filepath).await,
    //     FileType::Cpp => cpp(filepath).await,
    //     FileType::Java => java(filepath).await,
    //     FileType::CSharp => csharp(filepath).await,
    //     _ => Err(anyhow::anyhow!(DockerError::UnsupportedRuntime(
    //         ft.to_string()
    //     ))),
    // }
}

// pub async fn node(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/file.js");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "node:alpine",
//         "node",
//         "/root/app/file.js"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn rust(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/main.rs");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "rust:alpine",
//         "sh",
//         "-c",
//         "rustc /root/app/main.rs && /root/app/main"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn python(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/file.py");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "python:alpine",
//         "python",
//         "/root/app/file.py"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn ruby(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/file.rb");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "ruby:alpine",
//         "ruby",
//         "/root/app/file.rb"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn go(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/main.go");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "golang:alpine",
//         "go",
//         "run",
//         "/root/app/main.go"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn php(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/file.php");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "php:alpine",
//         "php",
//         "/root/app/file.php"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn perl(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/file.pl");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "perl",
//         "perl",
//         "/root/app/file.pl"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn bash(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/file.sh");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "bash:alpine",
//         "sh",
//         "/root/app/file.sh"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn c(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/main.c");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "gcc:alpine",
//         "sh",
//         "-c",
//         "gcc /root/app/main.c -o /root/app/main && /root/app/main"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn cpp(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/main.cpp");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "gcc:alpine",
//         "sh",
//         "-c",
//         "g++ /root/app/main.cpp -o /root/app/main && /root/app/main"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn java(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/Main.java");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "openjdk:alpine",
//         "sh",
//         "-c",
//         "javac /root/app/Main.java && java -cp /root/app Main"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
//
// pub async fn csharp(filepath: &str) -> anyhow::Result<()> {
//     let volume = format!("{filepath}:/root/app/Program.cs");
//
//     spawn_command!(
//         "docker",
//         "run",
//         "-v",
//         &volume,
//         "-it",
//         "mcr.microsoft.com/dotnet/core/sdk:alpine",
//         "sh",
//         "-c",
//         "dotnet run --project /root/app"
//     )?
//     .wait()
//     .await?;
//
//     Ok(())
// }
