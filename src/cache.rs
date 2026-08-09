use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use dirs::cache_dir;
use serde::{Deserialize, Serialize};

use crate::font_scraper::{Font, FontScraper};

const CACHE_FILE: &str = "nerd_fonts.json";
const DEFAULT_TTL_HOURS: i64 = 24 * 7;

#[derive(Deserialize, Serialize, Debug)]
pub struct FontCache {
    pub last_update: DateTime<Utc>,
    pub fonts: Vec<Font>,
}

impl FontCache {
    pub fn path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = cache_dir()
            .ok_or("could not determine cache directory")?
            .join("nf-installer");
        fs::create_dir_all(&dir)?;
        Ok(dir.join(CACHE_FILE))
    }

    pub fn load() -> Option<Self> {
        let raw = fs::read_to_string(Self::path().ok()?).ok()?;
        serde_json::from_str(&raw).ok()
    }

    pub fn store(fonts: Vec<Font>) -> Result<Self, Box<dyn std::error::Error>> {
        let cache = Self {
            last_update: Utc::now(),
            fonts,
        };
        fs::write(Self::path()?, serde_json::to_string_pretty(&cache)?)?;
        Ok(cache)
    }

    pub fn is_stale(&self, ttl_hours: i64) -> bool {
        Utc::now() - self.last_update > Duration::hours(ttl_hours)
    }

    pub async fn get(force_refresh: bool) -> Result<Vec<Font>, Box<dyn std::error::Error>> {
        let mut cached = Self::load();

        if !force_refresh && let Some(cache) = cached.take_if(|c| !c.is_stale(DEFAULT_TTL_HOURS)) {
            return Ok(cache.fonts);
        }

        match FontScraper::get_font_names().await {
            Ok(fonts) => Ok(Self::store(fonts)?.fonts),
            Err(e) => match cached {
                Some(cache) => {
                    eprintln!("Update failed, using cache: {e}");
                    Ok(cache.fonts)
                }
                None => Err(e),
            },
        }
    }
}
