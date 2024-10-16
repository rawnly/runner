use crate::file_type::FileType;
use std::{env, path::PathBuf, process};
use tokio::io::AsyncWriteExt;

pub fn clear_screen() {
    eprint!("\x1B[2J\x1B[1;1H");
}

pub async fn temp_file(ft: &FileType) -> anyhow::Result<PathBuf> {
    let pid = process::id();

    let mut path = env::temp_dir();
    path.push(&format!("runner-{pid}.{}", ft.get_extension()));

    let mut file = tokio::fs::File::create(&path).await?;
    file.write_all(ft.get_template().as_bytes()).await?;

    Ok(path)
}
