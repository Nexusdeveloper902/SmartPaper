use anyhow::Result;
use shared::MediaType;
use std::path::Path;
use tokio::process::{Child, Command};

pub struct WallpaperManager {
    current_video_process: Option<Child>,
    current_path: Option<std::path::PathBuf>,
}

impl WallpaperManager {
    pub fn new() -> Self {
        Self {
            current_video_process: None,
            current_path: None,
        }
    }

    pub async fn set_wallpaper(&mut self, path: &std::path::Path, media_type: &MediaType) -> Result<()> {
        if self.current_path.as_deref() == Some(path) {
            println!("Wallpaper already set to {:?}, skipping", path);
            return Ok(());
        }

        println!("Setting wallpaper: {:?} ({:?})", path, media_type);
        // Kill existing video process if any
        if let Some(mut child) = self.current_video_process.take() {
            let _ = child.kill().await;
        }

        match media_type {
            MediaType::Image => {
                // Use the user's native switchwall script for images
                // Run inside a login shell to ensure all environment variables are sourced
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/jperez".to_string());
                let script_path = format!("{}/.config/quickshell/ii/scripts/colors/switchwall.sh", home);
                
                let status = Command::new("bash")
                    .arg("-l")
                    .arg("-c")
                    .arg(format!("\"{}\" \"{}\"", script_path, path.display()))
                    .stdin(std::process::Stdio::null())
                    .status()
                    .await?;
                
                if !status.success() {
                    eprintln!("switchwall.sh failed with status: {}. Is the graphical environment loaded?", status);
                }
            }
            MediaType::Video => {
                // mpvpaper '*' <path> -o "loop"
                let child = Command::new("/usr/bin/mpvpaper")
                    .arg("*")
                    .arg(path)
                    .arg("-o")
                    .arg("loop=inf")
                    .stdin(std::process::Stdio::null())
                    .spawn()?;
                self.current_video_process = Some(child);
            }
        }

        self.current_path = Some(path.to_path_buf());
        Ok(())
    }
}
