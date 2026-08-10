use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use zip::ZipArchive;

pub const FONT_EXTENSIONS: [&str; 2] = ["ttf", "otf"];
pub const WEB_EXTENSIONS: [&str; 1] = ["woff2"];

const ARCHIVE_EXTENSION: &str = "zip";
const WRITE_BUFFER_SIZE: usize = 64 * 1024;
const EXTRACT_TEMPLATE: &str = "{msg:<12} [{bar:30}] {pos}/{len}";
const PROGRESS_CHARS: &str = "=> ";

/// Extracts all matching files from the archive, ignoring the directory layout inside it.
pub fn extract(
    archive_path: &Path,
    target: &Path,
    extensions: &[&str],
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut archive = ZipArchive::new(File::open(archive_path)?)?;
    let entries = archive.len();
    let mut extracted = 0;

    let bar = ProgressBar::new(entries as u64);
    bar.set_style(ProgressStyle::with_template(EXTRACT_TEMPLATE)?.progress_chars(PROGRESS_CHARS));
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

        if !has_extension(&name, extensions) {
            continue;
        }

        let mut out =
            BufWriter::with_capacity(WRITE_BUFFER_SIZE, File::create(target.join(&name))?);
        io::copy(&mut entry, &mut out)?;
        out.flush()?;
        extracted += 1;
    }

    bar.finish_with_message("Extracted");

    Ok(extracted)
}

/// Copies font files from a single file or a directory into the target.
pub fn copy_fonts(source: &Path, target: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    if source.is_file() {
        return copy_font_file(source, target);
    }

    let mut copied = 0;

    for entry in fs::read_dir(source)? {
        let path = entry?.path();

        if path.is_file() {
            copied += copy_font_file(&path, target)?;
        }
    }

    Ok(copied)
}

/// Copies a single file if it carries a font extension.
fn copy_font_file(path: &Path, target: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(name) = path.file_name() else {
        return Ok(0);
    };

    if !has_extension(name, &FONT_EXTENSIONS) {
        return Ok(0);
    }

    fs::copy(path, target.join(name))?;

    Ok(1)
}

/// Checks whether the path points to a zip archive.
pub fn is_archive(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(ARCHIVE_EXTENSION))
}

/// Checks whether the file name carries one of the given extensions.
fn has_extension(name: &OsStr, extensions: &[&str]) -> bool {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| extensions.contains(&ext.to_lowercase().as_str()))
}
