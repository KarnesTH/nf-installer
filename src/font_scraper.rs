use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};

#[derive(Deserialize, Serialize, Debug)]
pub struct Font {
    pub name: String,
    pub download_url: String,
}

pub struct FontScraper {}

impl FontScraper {
    pub async fn get_font_names() -> Result<Vec<Font>, Box<dyn std::error::Error>> {
        let url = "https://www.nerdfonts.com/font-downloads";
        let html = Self::fetch_html(url).await.unwrap();
        let fonts = Self::extract_fonts(&html)?;

        Ok(fonts)
    }

    async fn fetch_html(url: &str) -> Result<String, reqwest::Error> {
        reqwest::get(url)
            .await?
            .error_for_status()?
            .text()
            .await
    }

    fn extract_fonts(html: &str) -> Result<Vec<Font>, Box<dyn std::error::Error>> {
        let document = Html::parse_document(html);

        let names = document.select(&Selector::parse("span.nerd-font-invisible-text").unwrap()).map(|node| node.text().collect::<String>()).collect::<Vec<String>>();
        let download_urls = document.select(&Selector::parse("a.nf-fa-download").unwrap()).map(|node| node.value().attr("href").unwrap().to_string()).collect::<Vec<String>>();

        let fonts: Vec<Font> = names.iter().zip(download_urls.iter()).map(|(name, download_url)| Font { name: name.clone(), download_url: download_url.clone() }).collect();

        Ok(fonts)
    }
}