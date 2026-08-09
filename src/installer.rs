use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use indicatif::{ProgressBar, ProgressStyle};
use zip::ZipArchive;

use crate::font_scraper::Font;

const FONT_EXTENSIONS: [&str; 2] = ["ttf", "otf"];
const WRITE_BUFFER_SIZE: usize = 64 * 1024;

const DOWNLOAD_TEMPLATE: &str =
    "{msg:<12} [{bar:30}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})";
const SPINNER_TEMPLATE: &str = "{msg:<12} {spinner} {bytes} ({bytes_per_sec})";
const EXTRACT_TEMPLATE: &str = "{msg:<12} [{bar:30}] {pos}/{len}";
const PROGRESS_CHARS: &str = "=> ";

pub struct FontInstaller;

impl FontInstaller {
    pub async fn install(font: &Font) -> Result<usize, Box<dyn std::error::Error>> {
        let archive_path = Self::download(font).await?;
        let target = Self::target_dir(font)?;

        let result = Self::extract(&archive_path, &target);
        let _ = fs::remove_file(&archive_path);
        let installed = result?;

        println!("Installed to {}", target.display());
        Self::refresh_font_cache()?;

        Ok(installed)
    }

    pub fn uninstall(name: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let dir = Self::font_root()?.join(name);

        if !dir.is_dir() {
            return Err(format!("{name} is not installed").into());
        }

        let removed = fs::read_dir(&dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .count();

        fs::remove_dir_all(&dir)?;
        println!("Removed {}", dir.display());
        Self::refresh_font_cache()?;

        Ok(removed)
    }

    pub fn installed() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let root = Self::font_root()?;

        if !root.is_dir() {
            return Ok(Vec::new());
        }

        let mut names: Vec<String> = fs::read_dir(root)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        names.sort();

        Ok(names)
    }

    async fn download(font: &Font) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut response = reqwest::get(&font.download_url).await?.error_for_status()?;

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

        let path = std::env::temp_dir().join(format!("nf-installer-{}.zip", font.name));
        let mut file = BufWriter::with_capacity(WRITE_BUFFER_SIZE, File::create(&path)?);

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk)?;
            bar.inc(chunk.len() as u64);
        }

        file.flush()?;
        bar.finish_with_message("Downloaded");

        Ok(path)
    }

    fn font_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(dirs::font_dir().ok_or("could not determine font directory")?)
    }

    fn target_dir(font: &Font) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = Self::font_root()?.join(&font.name);
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }

    fn extract(archive_path: &Path, target: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        let mut archive = ZipArchive::new(File::open(archive_path)?)?;
        let entries = archive.len();
        let mut installed = 0;

        let bar = ProgressBar::new(entries as u64);
        bar.set_style(
            ProgressStyle::with_template(EXTRACT_TEMPLATE)?.progress_chars(PROGRESS_CHARS),
        );
        bar.set_message("Extracting");

        for i in 0..entries {
            let mut entry = archive.by_index(i)?;
            bar.inc(1);

            let Some(name) = entry
                .enclosed_name()
                .and_then(|path| path.file_name().map(|name| name.to_owned()))
            else {
                continue;
            };

            let is_font = Path::new(&name)
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| FONT_EXTENSIONS.contains(&ext.to_lowercase().as_str()));

            if !is_font {
                continue;
            }

            let mut out =
                BufWriter::with_capacity(WRITE_BUFFER_SIZE, File::create(target.join(&name))?);
            io::copy(&mut entry, &mut out)?;
            out.flush()?;
            installed += 1;
        }

        bar.finish_with_message("Extracted");

        Ok(installed)
    }

    fn refresh_font_cache() -> Result<(), Box<dyn std::error::Error>> {
        println!("Refreshing font cache...");

        match Command::new("fc-cache").arg("-f").arg("-v").status() {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(format!("fc-cache exited with {status}").into()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                eprintln!("fc-cache not found, skipping font cache refresh");
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}
