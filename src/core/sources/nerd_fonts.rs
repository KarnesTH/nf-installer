use std::path::PathBuf;

use scraper::{Html, Selector};

use crate::core::sources::{Font, FontSource};
use crate::utils::cache::Cache;
use crate::utils::download;

const URL: &str = "https://www.nerdfonts.com/font-downloads";
const CACHE_NAME: &str = "nerd_fonts";

pub struct NerdFonts;

impl FontSource for NerdFonts {
    async fn list(&self, refresh: bool) -> Result<Vec<Font>, Box<dyn std::error::Error>> {
        Cache::get(CACHE_NAME, refresh, Self::scrape).await
    }

    async fn fetch(&self, font: &Font) -> Result<PathBuf, Box<dyn std::error::Error>> {
        download::to_temp(
            &font.download_url,
            &format!("nf-installer-{}.zip", font.name),
        )
        .await
    }
}

impl NerdFonts {
    /// Fetches the download page and reads the font list from it.
    async fn scrape() -> Result<Vec<Font>, Box<dyn std::error::Error>> {
        let html = reqwest::get(URL).await?.error_for_status()?.text().await?;

        Self::extract_fonts(&html)
    }

    /// Reads names and download URLs out of the page markup.
    fn extract_fonts(html: &str) -> Result<Vec<Font>, Box<dyn std::error::Error>> {
        let document = Html::parse_document(html);

        let names = document
            .select(&Selector::parse("span.nerd-font-invisible-text")?)
            .map(|node| node.text().collect::<String>().trim().to_string())
            .collect::<Vec<String>>();
        let download_urls = document
            .select(&Selector::parse("a.nf-fa-download")?)
            .filter_map(|node| node.value().attr("href").map(|href| href.to_string()))
            .collect::<Vec<String>>();

        let fonts = names
            .into_iter()
            .zip(download_urls)
            .map(|(name, download_url)| Font { name, download_url })
            .collect();

        Ok(fonts)
    }
}
