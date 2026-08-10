use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};

const WRITE_BUFFER_SIZE: usize = 64 * 1024;
const DOWNLOAD_TEMPLATE: &str =
    "{msg:<12} [{bar:30}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})";
const SPINNER_TEMPLATE: &str = "{msg:<12} {spinner} {bytes} ({bytes_per_sec})";
const PROGRESS_CHARS: &str = "=> ";

/// Streams the given URL into the system temp directory and returns the file path.
pub async fn to_temp(url: &str, file_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut response = reqwest::get(url).await?.error_for_status()?;

    let bar = match response.content_length() {
        Some(total) => {
            let bar = ProgressBar::new(total);
            bar.set_style(
                ProgressStyle::with_template(DOWNLOAD_TEMPLATE)?.progress_chars(PROGRESS_CHARS),
            );
            bar
        }
        None => {
            let bar = ProgressBar::new_spinner();
            bar.set_style(ProgressStyle::with_template(SPINNER_TEMPLATE)?);
            bar
        }
    };
    bar.set_message("Downloading");

    let path = std::env::temp_dir().join(file_name);
    let mut file = BufWriter::with_capacity(WRITE_BUFFER_SIZE, File::create(&path)?);

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
        bar.inc(chunk.len() as u64);
    }

    file.flush()?;
    bar.finish_with_message("Downloaded");

    Ok(path)
}
