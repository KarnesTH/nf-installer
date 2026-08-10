use std::path::PathBuf;

use serde::Deserialize;

use crate::core::sources::{Font, FontSource};
use crate::utils::cache::Cache;
use crate::utils::download;

const API_URL: &str = "https://gwfh.mranftl.com/api/fonts";
const CACHE_NAME: &str = "google_fonts";
const DEFAULT_SUBSETS: &str = "latin,latin-ext";

/// Format requested from the API, depending on where the font is going.
pub enum Format {
    /// Installed into the system.
    System,
    /// Written into a web project.
    Web,
}

impl Format {
    fn as_str(&self) -> &'static str {
        match self {
            Self::System => "ttf",
            Self::Web => "woff2",
        }
    }
}

pub struct GoogleFonts {
    format: Format,
    variants: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ApiFont {
    id: String,
    family: String,
}

impl FontSource for GoogleFonts {
    async fn list(&self, refresh: bool) -> Result<Vec<Font>, Box<dyn std::error::Error>> {
        Cache::get(CACHE_NAME, refresh, Self::families).await
    }

    async fn fetch(&self, font: &Font) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut url = format!(
            "{}?download=zip&subsets={}&formats={}",
            font.download_url,
            DEFAULT_SUBSETS,
            self.format.as_str()
        );

        if let Some(variants) = &self.variants {
            url.push_str(&format!("&variants={}", variants.join(",")));
        }

        download::to_temp(&url, &format!("nf-installer-{}.zip", font.name)).await
    }
}

impl GoogleFonts {
    /// Creates a source that fetches files suitable for a system installation.
    pub fn system(variants: Option<Vec<String>>) -> Self {
        Self {
            format: Format::System,
            variants,
        }
    }

    /// Creates a source that fetches woff2 files for a web project.
    pub fn web(variants: Option<Vec<String>>) -> Self {
        Self {
            format: Format::Web,
            variants,
        }
    }

    /// Fetches the list of available families from the API.
    async fn families() -> Result<Vec<Font>, Box<dyn std::error::Error>> {
        let api_fonts: Vec<ApiFont> = reqwest::get(API_URL)
            .await?
            .error_for_status()?
            .json()
            .await?;

        let fonts = api_fonts
            .into_iter()
            .map(|api_font| Font {
                name: api_font.family,
                download_url: format!("{API_URL}/{}", api_font.id),
            })
            .collect();

        Ok(fonts)
    }
}
