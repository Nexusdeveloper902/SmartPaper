use anyhow::{Context, Result};
use shared::MediaType;
use std::path::Path;
use tokio::process::Command;

pub struct WallpaperManager {
    current_path: Option<std::path::PathBuf>,
}

impl WallpaperManager {
    pub fn new() -> Self {
        Self {
            current_path: None,
        }
    }

    pub async fn set_wallpaper(&mut self, path: &std::path::Path, media_type: &MediaType) -> Result<()> {
        if self.current_path.as_deref() == Some(path) {
            println!("Wallpaper already set to {:?}, skipping", path);
            return Ok(());
        }

        println!("Setting wallpaper via switchwall.sh: {:?} ({:?})", path, media_type);

        // Resolve the switchwall.sh script path relative to user home
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let script_path = Path::new(&home).join(".config/quickshell/ii/scripts/colors/switchwall.sh");

        if !script_path.exists() {
            anyhow::bail!("switchwall.sh not found at {:?}", script_path);
        }
        
        // Spawn switchwall.sh. We deliberately detach from it (no await)
        // to avoid blocking the daemon loop while the script generates themes.
        let mut child = Command::new(&script_path)
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())  // suppress subprocess chatter from daemon's log
            .stderr(std::process::Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to spawn switchwall.sh for {:?}", path))?;

        // Detach: spawn a brief async task to log if the child fails quickly
        tokio::spawn(async move {
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                child.wait(),
            ).await;
            match result {
                Ok(Ok(status)) if !status.success() => {
                    eprintln!("switchwall.sh finished with error status: {}", status);
                }
                Ok(Ok(_)) => {} // success
                Ok(Err(e)) => {
                    eprintln!("switchwall.sh wait error: {}", e);
                }
                Err(_) => {
                    // Timed out after 3s — script is still running, that's normal
                }
            }
        });

        self.current_path = Some(path.to_path_buf());
        Ok(())
    }
}
