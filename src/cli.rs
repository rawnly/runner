use crate::file_type::FileType;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// path to the file to watch
    pub path: Option<PathBuf>,

    /// runtime to use for the file
    /// if not provided, it will be inferred from the file extension
    #[clap(long)]
    pub runtime: Option<FileType>,

    /// command to run when the file changes -
    /// if includes whitespace, it will be split and the first part will be the command
    /// when using docker you can use {entrypoint} to refer to the executable
    #[clap(long)]
    pub command: Option<String>,

    #[clap(long)]
    pub image: Option<String>,

    /// environment variables to pass to the command
    /// e.g. `--env "KEY=VALUE"`
    #[clap(short)]
    pub env: Option<Vec<String>>,

    /// do not use docker to run the code
    /// this is useful when you want to run the code on your local machine
    #[clap(long)]
    pub no_docker: bool,
}
