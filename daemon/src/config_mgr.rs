use anyhow::{Context, Result};
use directories::ProjectDirs;
use shared::Config;
use std::path::PathBuf;
use tokio::fs;

pub fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "smart-wallpaper", "app")
        .context("Could not find project directories")?;
    let config_dir = proj_dirs.config_dir();
    Ok(config_dir.join("config.json"))
}

pub async fn load_config() -> Result<Config> {
    let path = get_config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let data = fs::read_to_string(&path).await?;
    let config: Config = serde_json::from_str(&data)?;
    Ok(config)
}

pub async fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let data = serde_json::to_string_pretty(config)?;
    fs::write(&path, data).await?;
    Ok(())
}
