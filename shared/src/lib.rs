use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaItem {
    pub path: PathBuf,
    pub media_type: MediaType,
    pub included: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub wallpaper_dir: Option<PathBuf>,
    pub interval_seconds: u64,
    pub media: Vec<MediaItem>,
    #[serde(default)]
    pub random: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallpaper_dir: None,
            interval_seconds: 300, // 5 minutes
            media: Vec::new(),
            random: true,
        }
    }
}
