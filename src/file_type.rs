use crate::{command::docker::DockerImage, templates};
use anyhow::anyhow;
use std::{env, path::PathBuf};
use tokio::process::Command;

#[derive(Debug, clap::ValueEnum, Clone)]
pub enum FileType {
    Perl,
    Php,
    Ruby,
    C,
    Cpp,
    Python,
    Python3,
    Node,
    Go,
    Typescript,
    CSharp,
    Java,
    Swift,
    Scala,
    Rust,
    Shell,
    Unsupported,
}

impl FileType {
    pub fn get_docker_image(&self) -> Option<DockerImage> {
        match self {
            Self::Node => Some(DockerImage::alpine("node")),
            Self::Ruby => Some(DockerImage::alpine("ruby")),
            Self::Go => Some(DockerImage::alpine("golang")),
            Self::Php => Some(DockerImage::alpine("php")),
            Self::Perl => Some(DockerImage::latest("perl")),
            Self::Java => Some(DockerImage::alpine("openjdk")),
            _ => None,
        }
    }

    pub fn get_docker_command(&self) -> Option<String> {
        let entrypoint = self.get_docker_entrypoint();

        if let Some(entrypoint) = entrypoint {
            return match self {
                Self::Java => Some(format!(
                    "java /root/app/{entrypoint} && java -cp /root/app Main"
                )),
                Self::Node => Some(format!("node /root/app/{entrypoint}")),
                Self::Python | Self::Python3 => Some(format!("python3 /root/app/{entrypoint}")),
                Self::Go => Some(format!("go run /root/app/{entrypoint}")),
                Self::Typescript => Some(format!(
                    "tsc /root/app/{entrypoint} && node /root/app/main.js"
                )),
                Self::Rust => Some(format!("rustc /root/app/{entrypoint} && /root/app/main")),
                Self::Shell => Some(format!("bash /root/app/{entrypoint}")),
                Self::Ruby => Some(format!("ruby /root/app/{entrypoint}")),
                Self::Php => Some(format!("php /root/app/{entrypoint}")),
                Self::Perl => Some(format!("perl /root/app/{entrypoint}")),
                Self::C => Some(format!(
                    "gcc /root/app/{entrypoint} -o /root/app/main && /root/app/main"
                )),
                Self::Cpp => Some(format!(
                    "g++ /root/app/{entrypoint} -o /root/app/main && /root/app/main"
                )),
                Self::CSharp => Some(format!(
                    "csc /root/app/{entrypoint} && mono /root/app/main.exe"
                )),
                _ => None,
            };
        }

        None
    }

    pub fn get_docker_entrypoint(&self) -> Option<String> {
        match self {
            Self::Java => Some(format!("Main.{}", self.get_extension())),
            Self::Unsupported => None,
            _ => Some(format!("main.{}", self.get_extension())),
        }
    }

    pub fn get_template(&self) -> String {
        match self {
            Self::Java => templates::JAVA.to_string(),
            Self::Swift => templates::SWIFT.to_string(),
            Self::Scala => templates::SCALA.to_string(),
            Self::CSharp => templates::CSHARP.to_string(),
            Self::Php => templates::PHP.to_string(),
            Self::Ruby => templates::RUBY.to_string(),
            Self::C => templates::C.to_string(),
            Self::Cpp => templates::CPP.to_string(),
            Self::Perl => templates::PERL.to_string(),
            Self::Python | Self::Python3 => templates::PYTHON.to_string(),
            Self::Go => templates::GO.to_string(),
            Self::Typescript | Self::Node => templates::NODE.to_string(),
            Self::Rust => templates::RUST.to_string(),
            Self::Shell => templates::BASH.to_string(),
            Self::Unsupported => "".to_string(),
        }
    }

    pub fn get_extension(&self) -> String {
        match self {
            Self::Perl => "pl",
            Self::Python => "py",
            Self::Python3 => "py",
            Self::Node => "js",
            Self::Go => "go",
            Self::Typescript => "ts",
            Self::Rust => "rs",
            Self::Shell => "sh",
            Self::Ruby => "rb",
            Self::Php => "php",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Java => "java",
            Self::Swift => "swift",
            Self::Scala => "scala",
            Self::CSharp => "cs",
            Self::Unsupported => "unsupported",
        }
        .to_string()
    }
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
            "ts" => Ok(Self::Typescript),
            "rs" => Ok(Self::Rust),
            "pl" => Ok(Self::Perl),
            "php" => Ok(Self::Php),
            "rb" => Ok(Self::Ruby),
            "c" => Ok(Self::C),
            "cpp" => Ok(Self::Cpp),
            "java" => Ok(Self::Java),
            "swift" => Ok(Self::Swift),
            "scala" => Ok(Self::Scala),
            "cs" => Ok(Self::CSharp),
            _ => Ok(Self::Unsupported),
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
            Self::Typescript => "bun",
            Self::Rust => "rustc",
            Self::Perl => "perl",
            Self::Php => "php",
            Self::Ruby => "ruby",
            Self::C => "gcc",
            Self::Cpp => "g++",
            Self::Java => "javac",
            Self::Swift => "swiftc",
            Self::Scala => "scalac",
            Self::CSharp => "csc",
            _ => "Unsupported",
        }
        .to_string()
    }
}

impl FileType {
    pub async fn is_available(&self) -> Result<bool, anyhow::Error> {
        match self {
            Self::Unsupported => Err(anyhow!(
                "cannot check availability for unsupported file type"
            )),
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

    pub async fn get_command_name(&self) -> Result<String, anyhow::Error> {
        match self {
            Self::Unsupported => Err(anyhow!("cannot get command name for unsupported file type")),
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
