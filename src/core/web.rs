use std::fs;
use std::path::Path;

use crate::core::sources::archive;

const CSS_FILE: &str = "fonts.css";
const DEFAULT_WEIGHT: &str = "400";
const ITALIC_SUFFIX: &str = "italic";

const LICENSE_BASE_URL: &str = "https://raw.githubusercontent.com/google/fonts/main";
const LICENSE_LOCATIONS: [(&str, &str); 3] = [
    ("ofl", "OFL.txt"),
    ("apache", "LICENSE.txt"),
    ("ufl", "UFL.txt"),
];

pub struct WebAssets;

impl WebAssets {
    /// Extracts the woff2 files into the output directory and writes a stylesheet and the
    /// font license next to them. The downloaded archive is removed afterward.
    pub async fn write(
        family: &str,
        archive_path: &Path,
        out_dir: &Path,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        fs::create_dir_all(out_dir)?;

        let result = archive::extract(archive_path, out_dir, &archive::WEB_EXTENSIONS);
        let _ = fs::remove_file(archive_path);
        let extracted = result?;

        if extracted == 0 {
            return Err("No woff2 files in the archive".into());
        }

        fs::write(out_dir.join(CSS_FILE), Self::stylesheet(family, out_dir)?)?;
        println!("Wrote {CSS_FILE} and {extracted} font files");

        Self::write_license(family, out_dir).await?;

        Ok(extracted)
    }

    /// Downloads the license of the family and writes it next to the font files.
    /// Fonts are redistributed with this, so the license has to travel with them.
    async fn write_license(family: &str, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let slug = family.to_lowercase().replace(' ', "");

        for (dir, file) in LICENSE_LOCATIONS {
            let url = format!("{LICENSE_BASE_URL}/{dir}/{slug}/{file}");
            let response = reqwest::get(&url).await?;

            if !response.status().is_success() {
                continue;
            }

            fs::write(out_dir.join(file), response.text().await?)?;
            println!("Wrote {file}");

            return Ok(());
        }

        eprintln!("Warning: no license file found for {family}");
        eprintln!("Check the license on fonts.google.com before shipping the files");

        Ok(())
    }

    /// Builds a stylesheet with one `@font-face` rule per woff2 file in the directory.
    fn stylesheet(family: &str, dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let mut files: Vec<String> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("woff2"))
            })
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        files.sort();

        let faces: Vec<String> = files.iter().map(|file| Self::face(family, file)).collect();

        Ok(faces.join("\n\n"))
    }

    /// Builds a single `@font-face` rule for the given file.
    fn face(family: &str, file: &str) -> String {
        let (weight, style) = Self::variant_of(file);

        format!(
            "@font-face {{\n  \
             font-family: '{family}';\n  \
             font-style: {style};\n  \
             font-weight: {weight};\n  \
             font-display: swap;\n  \
             src: url('./{file}') format('woff2');\n}}"
        )
    }

    /// Reads weight and style out of the file name, which ends in the variant
    /// as named by the API, e.g. `roboto-v30-latin-700italic.woff2`.
    fn variant_of(file: &str) -> (String, &'static str) {
        let stem = file.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(file);
        let variant = stem.rsplit('-').next().unwrap_or(DEFAULT_WEIGHT);

        let (digits, style) = match variant.strip_suffix(ITALIC_SUFFIX) {
            Some(digits) => (digits, "italic"),
            None => (variant, "normal"),
        };

        let weight = if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
            digits.to_string()
        } else {
            DEFAULT_WEIGHT.to_string()
        };

        (weight, style)
    }
}
