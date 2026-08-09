use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use zip::ZipArchive;

use crate::font_scraper::Font;

const FONT_EXTENSIONS: [&str; 2] = ["ttf", "otf"];

pub struct FontInstaller;

impl FontInstaller {
    pub async fn install(font: &Font) -> Result<usize, Box<dyn std::error::Error>> {
        let archive_path = Self::download(font).await?;
        let target = Self::target_dir(font)?;

        let result = Self::extract(&archive_path, &target);
        let _ = fs::remove_file(&archive_path);
        let installed = result?;

        Self::refresh_font_cache()?;

        Ok(installed)
    }

    async fn download(font: &Font) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let bytes = reqwest::get(&font.download_url)
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let path = std::env::temp_dir().join(format!("nf-installer-{}.zip", font.name));
        fs::write(&path, &bytes)?;

        Ok(path)
    }

    fn target_dir(font: &Font) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = dirs::font_dir()
            .ok_or("could not determine font directory")?
            .join(&font.name);
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }

    fn extract(archive_path: &Path, target: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        let mut archive = ZipArchive::new(File::open(archive_path)?)?;
        let mut installed = 0;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;

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

            let mut out = File::create(target.join(&name))?;
            io::copy(&mut entry, &mut out)?;
            installed += 1;
        }

        Ok(installed)
    }

    fn refresh_font_cache() -> Result<(), Box<dyn std::error::Error>> {
        match Command::new("fc-cache").arg("-f").status() {
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
