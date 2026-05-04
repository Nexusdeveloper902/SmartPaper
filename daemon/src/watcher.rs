use anyhow::Result;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use shared::{Config, MediaItem, MediaType};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio::task;

pub enum WatchEvent {
    ConfigChanged,
    DirEvent,
}

pub fn start_watchers(
    config_path: PathBuf,
    wallpaper_dir: Option<PathBuf>,
    tx: mpsc::Sender<WatchEvent>,
) -> Result<RecommendedWatcher> {
    let (std_tx, std_rx) = std::sync::mpsc::channel();
    
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = std_tx.send(event);
            }
        },
        NotifyConfig::default(),
    )?;

    // Watch config file
    if let Some(parent) = config_path.parent() {
        let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
    }

    // Watch wallpaper directory
    if let Some(dir) = wallpaper_dir {
        let _ = watcher.watch(&dir, RecursiveMode::NonRecursive);
    }

    // Forward events to tokio channel
    task::spawn_blocking(move || {
        for event in std_rx {
            use notify::EventKind;
            // Simplified handling: just send generic event types
            let mut is_config = false;
            let mut is_dir = false;
            for path in &event.paths {
                if path == &config_path {
                    is_config = true;
                } else {
                    is_dir = true;
                }
            }
            if is_config {
                let _ = tx.blocking_send(WatchEvent::ConfigChanged);
            }
            if is_dir {
                // Only send DirEvent if it's a create/modify/remove
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        let _ = tx.blocking_send(WatchEvent::DirEvent);
                    }
                    _ => {}
                }
            }
        }
    });

    Ok(watcher)
}

pub fn scan_directory(dir: &Path) -> Result<Vec<MediaItem>> {
    let mut items = Vec::new();
    for entry in walkdir::WalkDir::new(dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let media_type = match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" | "png" | "webp" => Some(MediaType::Image),
                "mp4" | "webm" | "mkv" => Some(MediaType::Video),
                _ => None,
            };
            if let Some(mt) = media_type {
                items.push(MediaItem {
                    path,
                    media_type: mt,
                    included: true, // Default to included when discovered
                });
            }
        }
    }
    Ok(items)
}

pub fn sync_media_items(config: &mut Config) -> Result<bool> {
    let dir = match &config.wallpaper_dir {
        Some(d) => d,
        None => return Ok(false),
    };

    let discovered = scan_directory(dir)?;
    let mut changed = false;

    // Add new items
    for item in discovered {
        if !config.media.iter().any(|m| m.path == item.path) {
            config.media.push(item);
            changed = true;
        }
    }

    // Remove deleted items or items not in the current directory
    let original_len = config.media.len();
    config.media.retain(|m| m.path.exists() && m.path.parent() == Some(dir));
    if config.media.len() != original_len {
        changed = true;
    }

    Ok(changed)
}
