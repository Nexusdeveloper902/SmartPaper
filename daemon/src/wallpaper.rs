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

        println!("Setting wallpaper via switchwall.sh: {:?} ({:?})", path, media_type);

        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/jperez".to_string());
        let script_path = format!("{}/.config/quickshell/ii/scripts/colors/switchwall.sh", home);
        
        // Spawn switchwall.sh. We don't await its completion to avoid blocking the daemon loop
        // if the script takes a long time (e.g. generating themes or waiting for notifications).
        let mut child = Command::new("bash")
            .arg("-l")
            .arg("-c")
            .arg(format!("\"{}\" \"{}\"", script_path, path.display()))
            .stdin(std::process::Stdio::null())
            .spawn()?;

        // Give it a moment to start and check if it failed immediately
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Ok(Some(status)) = child.try_wait() {
            if !status.success() {
                eprintln!("switchwall.sh failed immediately with status: {}", status);
            }
        }

        self.current_path = Some(path.to_path_buf());
        Ok(())
    }
}
