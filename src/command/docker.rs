use std::{process::ExitStatus, str::FromStr};

use crate::{command, file_type::FileType, spawn_command};

#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("Unsupported docker runtime: {0}")]
    UnsupportedRuntime(String),

    #[error("Docker image does not exist: {0}")]
    ImageDoesNotExist(String),

    #[error("Docker image not installed: {0}")]
    ImageNotInstalled(String),

    #[error("Docker command failed: {0}")]
    CommandFailed(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
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

    async fn is_availble(&self) -> std::result::Result<bool, DockerError> {
        image_exists_on_machine(&self.get_image()).await
    }
}

impl FromStr for DockerImage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() == 1 {
            Ok(Self::new(parts[0], "latest"))
        } else {
            Ok(Self::new(parts[0], parts[1]))
        }
    }
}

impl ToString for DockerImage {
    fn to_string(&self) -> String {
        self.get_image()
    }
}

pub async fn pull(image: &str) -> std::result::Result<std::process::Output, DockerError> {
    Ok(command!("docker", "pull", image).output().await?)
}

pub async fn run(
    ft: &FileType,
    filepath: &str,
    docker_command: Option<String>,
    docker_image: Option<DockerImage>,
) -> std::result::Result<(ExitStatus, String), DockerError> {
    let image = docker_image
        .or(ft.get_docker_image())
        .ok_or(DockerError::UnsupportedRuntime(ft.to_string()))?;

    if !image.is_availble().await? {
        // ask the user if they want to pull the image
        return Err(DockerError::ImageNotInstalled(image.to_string()));
    }

    let entrypoint = ft
        .get_docker_entrypoint()
        .ok_or(DockerError::UnsupportedRuntime(ft.to_string()))?;

    let mut fp = filepath.to_string();
    if !filepath.starts_with("./") && !filepath.starts_with("/") {
        fp = format!("./{}", filepath);
    }

    let volume = format!("{fp}:/root/app/{entrypoint}");
    let command = docker_command
        .map(|c| {
            c.to_string()
                .replace("{entrypoint}", &format!("/root/app/{entrypoint}"))
        })
        .or(ft.get_docker_command())
        .ok_or(DockerError::UnsupportedRuntime(ft.to_string()))?;

    let image = image.to_string();

    let exit_status =
        spawn_command!("docker", "run", "-v", &volume, "-it", &image, "sh", "-c", &command)?
            .wait()
            .await?;

    Ok((exit_status, image))
}

pub async fn image_exists_on_machine(image: &str) -> std::result::Result<bool, DockerError> {
    let output = command!("docker", "image", "inspect", image)
        .output()
        .await?;

    Ok(output.status.success())
}
