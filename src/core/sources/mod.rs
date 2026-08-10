pub mod archive;
pub mod nerd_fonts;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Font {
    pub name: String,
    pub download_url: String,
}

impl std::fmt::Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A place fonts can be obtained from.
///
/// The lint is allowed because the trait is never used behind `dyn` and
/// therefore does not need an explicit `Send` bound.
#[allow(async_fn_in_trait)]
pub trait FontSource {
    /// Returns the available fonts, using a cached list when it is fresh enough.
    async fn list(&self, refresh: bool) -> Result<Vec<Font>, Box<dyn std::error::Error>>;

    /// Downloads the given font and returns the path of the downloaded archive.
    async fn fetch(&self, font: &Font) -> Result<PathBuf, Box<dyn std::error::Error>>;
}
