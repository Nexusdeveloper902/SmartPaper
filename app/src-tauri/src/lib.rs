use anyhow::{Context, Result};
use directories::ProjectDirs;
use shared::Config;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::process::Command;

fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "smart-wallpaper", "app")
        .context("Could not find project directories")?;
    let config_dir = proj_dirs.config_dir();
    Ok(config_dir.join("config.json"))
}

#[tauri::command]
async fn get_config() -> Result<Config, String> {
    let path = get_config_path().map_err(|e| e.to_string())?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let data = fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
    let config: Config = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
async fn save_config(config: Config) -> Result<(), String> {
    let path = get_config_path().map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, data).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn next_wallpaper() -> Result<(), String> {
    let socket_path = "/tmp/smart-wallpaper.sock";
    let mut stream = UnixStream::connect(socket_path).await.map_err(|e| e.to_string())?;
    stream.write_all(b"NEXT").await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn generate_thumbnail(video_path: String) -> Result<String, String> {
    let output_path = format!("{}.thumb.jpg", video_path);
    if !PathBuf::from(&output_path).exists() {
        // ffmpeg -i video.mp4 -ss 00:00:01 -vframes 1 thumb.jpg
        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(&video_path)
            .arg("-ss")
            .arg("00:00:01")
            .arg("-vframes")
            .arg("1")
            .arg(&output_path)
            .stdin(std::process::Stdio::null())
            .status()
            .await
            .map_err(|e| e.to_string())?;
            
        if !status.success() {
            return Err("Failed to generate thumbnail".into());
        }
    }
    Ok(output_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("NO_AT_BRIDGE", "1");
    std::env::set_var("GTK_A11Y", "none");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            next_wallpaper,
            generate_thumbnail
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
