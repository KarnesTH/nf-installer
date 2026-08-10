use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::sources::archive;

pub struct FontInstaller;

impl FontInstaller {
    /// Extracts a downloaded archive into the font directory and registers the font.
    /// The archive is removed afterward, including when the extraction fails.
    pub fn install_downloaded(
        name: &str,
        archive_path: &Path,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let target = Self::target_dir(name)?;

        let result = archive::extract(archive_path, &target, &archive::FONT_EXTENSIONS);
        let _ = fs::remove_file(archive_path);
        let installed = result?;

        Self::finish(&target)?;

        Ok(installed)
    }

    /// Installs fonts from a local archive, font file or directory. The source is left in place.
    pub fn install_local(name: &str, path: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        let target = Self::target_dir(name)?;

        let installed = if archive::is_archive(path) {
            archive::extract(path, &target, &archive::FONT_EXTENSIONS)?
        } else {
            archive::copy_fonts(path, &target)?
        };

        if installed == 0 {
            return Err("No font files found".into());
        }

        Self::finish(&target)?;

        Ok(installed)
    }

    /// Removes an installed font and refreshes the font cache. Returns the number of removed files.
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

    /// Lists the font directories that are present in the user font directory.
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

    /// Reports the target directory and refreshes the font cache.
    fn finish(target: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("Installed to {}", target.display());

        Self::refresh_font_cache()
    }

    /// Returns the user font directory.
    fn font_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(dirs::font_dir().ok_or("could not determine font directory")?)
    }

    /// Returns the directory the font files are installed into.
    fn target_dir(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = Self::font_root()?.join(name);
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }

    /// Rebuilds the fontconfig cache so the change is picked up.
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
