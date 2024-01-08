use tokio::fs::File;
use tokio::io::AsyncReadExt;
use anyhow::Result;
pub async fn read_file(file_path: &str) -> Result<String> {
    let mut f = File::open(file_path).await?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).await?;
    Ok(String::from_utf8(buffer)?)
}


