use std::fs;
use std::future::Future;
use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use dirs::cache_dir;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const CACHE_DIR: &str = "nf-installer";
const DEFAULT_TTL_HOURS: i64 = 24 * 7;

#[derive(Deserialize, Serialize, Debug)]
pub struct Cache<T> {
    pub last_update: DateTime<Utc>,
    pub items: Vec<T>,
}

impl<T: Serialize + DeserializeOwned> Cache<T> {
    /// Returns the items, fetching them only if the cache is missing, stale, or a refresh is forced.
    pub async fn get<F, Fut>(
        name: &str,
        refresh: bool,
        fetch: F,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output=Result<Vec<T>, Box<dyn std::error::Error>>>,
    {
        let mut cached = Self::load(name);

        if !refresh && let Some(cache) = cached.take_if(|cache| !cache.is_stale(DEFAULT_TTL_HOURS))
        {
            return Ok(cache.items);
        }

        match fetch().await {
            Ok(items) => Ok(Self::store(name, items)?.items),
            Err(e) => match cached {
                Some(cache) => {
                    eprintln!("Update failed, using cache: {e}");
                    Ok(cache.items)
                }
                None => Err(e),
            },
        }
    }

    /// Returns the cache file path, creating the parent directory if needed.
    pub fn path(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = cache_dir()
            .ok_or("could not determine cache directory")?
            .join(CACHE_DIR);
        fs::create_dir_all(&dir)?;

        Ok(dir.join(format!("{name}.json")))
    }

    /// Loads the cache from the disk. Returns `None` if it is missing or unreadable.
    pub fn load(name: &str) -> Option<Self> {
        let raw = fs::read_to_string(Self::path(name).ok()?).ok()?;

        serde_json::from_str(&raw).ok()
    }

    /// Writes the items to disk with the current timestamp.
    pub fn store(name: &str, items: Vec<T>) -> Result<Self, Box<dyn std::error::Error>> {
        let cache = Self {
            last_update: Utc::now(),
            items,
        };
        fs::write(Self::path(name)?, serde_json::to_string_pretty(&cache)?)?;

        Ok(cache)
    }

    /// Checks whether the cache is older than the given TTL.
    pub fn is_stale(&self, ttl_hours: i64) -> bool {
        Utc::now() - self.last_update > Duration::hours(ttl_hours)
    }
}
