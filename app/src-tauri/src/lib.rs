use anyhow::{Context, Result};
use directories::ProjectDirs;
use shared::Config;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

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
        fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, data).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn next_wallpaper() -> Result<(), String> {
    let socket_path = "/tmp/smart-wallpaper.sock";
    let mut stream = timeout(Duration::from_secs(2), UnixStream::connect(socket_path))
        .await
        .map_err(|_| "Timed out connecting to daemon".to_string())?
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    timeout(Duration::from_secs(2), stream.write_all(b"NEXT"))
        .await
        .map_err(|_| "Timed out sending command to daemon".to_string())?
        .map_err(|e| format!("Failed to send command: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn generate_thumbnail(video_path: String) -> Result<String, String> {
    // Sanitize: ensure the video path is absolute and canonical
    let video = Path::new(&video_path);
    if !video.is_absolute() {
        return Err("Video path must be absolute".into());
    }
    // Canonicalize to resolve any ".." or symlinks
    let video = fs::canonicalize(&video).await
        .map_err(|e| format!("Invalid video path: {}", e))?;

    let output_path = format!("{}.thumb.jpg", video.display());
    if !PathBuf::from(&output_path).exists() {
        // Check that ffmpeg is available before spawning
        let ffmpeg_check = Command::new("which")
            .arg("ffmpeg")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .map_err(|_| "Failed to check for ffmpeg".to_string())?;
        if !ffmpeg_check.success() {
            return Err("ffmpeg is not installed. Please install ffmpeg to generate thumbnails.".into());
        }

        // ffmpeg -i video.mp4 -ss 00:00:01 -vframes 1 thumb.jpg
        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(&video)
            .arg("-ss")
            .arg("00:00:01")
            .arg("-vframes")
            .arg("1")
            .arg(&output_path)
            .stdin(std::process::Stdio::null())
            .status()
            .await
            .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

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
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, shortcut, event| {
                    println!("Shortcut event: {:?} {:?}", shortcut, event.state());
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        if shortcut.matches(
                            tauri_plugin_global_shortcut::Modifiers::SUPER
                                | tauri_plugin_global_shortcut::Modifiers::ALT,
                            tauri_plugin_global_shortcut::Code::KeyN,
                        ) || shortcut.matches(
                            tauri_plugin_global_shortcut::Modifiers::META
                                | tauri_plugin_global_shortcut::Modifiers::ALT,
                            tauri_plugin_global_shortcut::Code::KeyN,
                        ) || shortcut.matches(
                            tauri_plugin_global_shortcut::Modifiers::CONTROL
                                | tauri_plugin_global_shortcut::Modifiers::ALT,
                            tauri_plugin_global_shortcut::Code::KeyJ,
                        ) {
                            println!("Global shortcut matches! Skipping to next wallpaper...");
                            tokio::spawn(async {
                                let _ = next_wallpaper().await;
                            });
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
                
                let shortcut_n = Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyN);
                match app.global_shortcut().register(shortcut_n) {
                    Ok(_) => println!("Hotkey (Super+Alt+N) registered successfully"),
                    Err(e) => eprintln!("Failed to register hotkey (Super+Alt+N): {}", e),
                }

                let shortcut_j = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyJ);
                match app.global_shortcut().register(shortcut_j) {
                    Ok(_) => println!("Hotkey (Ctrl+Alt+J) registered successfully"),
                    Err(e) => eprintln!("Failed to register hotkey (Ctrl+Alt+J): {}", e),
                }
            }
            Ok(())
        })
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
