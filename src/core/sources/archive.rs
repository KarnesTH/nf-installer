use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use zip::ZipArchive;

const FONT_EXTENSIONS: [&str; 2] = ["ttf", "otf"];
const WRITE_BUFFER_SIZE: usize = 64 * 1024;
const EXTRACT_TEMPLATE: &str = "{msg:<12} [{bar:30}] {pos}/{len}";
const PROGRESS_CHARS: &str = "=> ";

/// Extracts all font files from the archive, ignoring the directory layout inside it.
pub fn extract(archive_path: &Path, target: &Path) -> Result<usize, Box<dyn std::error::Error>> {
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

        if !is_font_file(&name) {
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

/// Checks whether the file name carries a font extension.
fn is_font_file(name: &std::ffi::OsStr) -> bool {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| FONT_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}
