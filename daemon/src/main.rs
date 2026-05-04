mod config_mgr;
mod ipc;
mod wallpaper;
mod watcher;

use anyhow::Result;
use rand::seq::SliceRandom;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

use wallpaper::WallpaperManager;
use watcher::WatchEvent;
use ipc::IpcCommand;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load config
    let mut config = config_mgr::load_config().await?;
    let config_path = config_mgr::get_config_path()?;

    // 2. Setup channels
    let (watch_tx, mut watch_rx) = mpsc::channel(32);
    let (ipc_tx, mut ipc_rx) = mpsc::channel(32);

    // Initial sync
    if watcher::sync_media_items(&mut config)? {
        config_mgr::save_config(&config).await?;
    }

    let mut _watcher = watcher::start_watchers(
        config_path.clone(),
        config.wallpaper_dir.clone(),
        watch_tx.clone(),
    )?;

    if let Err(e) = ipc::start_ipc_server(ipc_tx) {
        eprintln!("Failed to start IPC server: {}", e);
    }

    let mut wp_manager = WallpaperManager::new();

    let mut current_index = 0usize;

    // Helper closure to get included media
    let get_included_media = |cfg: &shared::Config| {
        let mut media = cfg.media
            .iter()
            .filter(|m| m.included)
            .cloned()
            .collect::<Vec<_>>();

        if cfg.random {
            let mut rng = rand::thread_rng();
            media.shuffle(&mut rng);
        } else {
            media.sort_by(|a, b| a.path.cmp(&b.path));
        }
        media
    };

    let mut included_media = get_included_media(&config);

    // Initial wallpaper
    if !included_media.is_empty() {
        let item = &included_media[0];
        let _ = wp_manager.set_wallpaper(&item.path, &item.media_type).await;
    }

    let mut next_tick = time::Instant::now() + Duration::from_secs(config.interval_seconds.max(1));

    loop {
        let sleep = time::sleep_until(next_tick);
        tokio::pin!(sleep);

        tokio::select! {
            _ = &mut sleep => {
                // Timer expired, next wallpaper
                next_tick = time::Instant::now() + Duration::from_secs(config.interval_seconds.max(1));
                if !included_media.is_empty() {
                    current_index = (current_index + 1) % included_media.len();
                    let item = &included_media[current_index];
                    if let Err(e) = wp_manager.set_wallpaper(&item.path, &item.media_type).await {
                        eprintln!("Failed to set wallpaper: {}", e);
                    }
                }
            }
            Some(cmd) = ipc_rx.recv() => {
                match cmd {
                    IpcCommand::Next => {
                        next_tick = time::Instant::now() + Duration::from_secs(config.interval_seconds.max(1));
                        if !included_media.is_empty() {
                            current_index = (current_index + 1) % included_media.len();
                            let item = &included_media[current_index];
                            if let Err(e) = wp_manager.set_wallpaper(&item.path, &item.media_type).await {
                                eprintln!("Failed to set wallpaper on Next: {}", e);
                            }
                        }
                    }
                }
            }
            Some(event) = watch_rx.recv() => {
                match event {
                    WatchEvent::ConfigChanged => {
                        // Reload config
                        if let Ok(mut new_config) = config_mgr::load_config().await {
                            let dir_changed = new_config.wallpaper_dir != config.wallpaper_dir;
                            
                            if dir_changed {
                                // Rescan new directory immediately
                                if let Ok(changed) = watcher::sync_media_items(&mut new_config) {
                                    if changed {
                                        let _ = config_mgr::save_config(&new_config).await;
                                    }
                                }
                                // Restart watcher
                                if let Ok(w) = watcher::start_watchers(
                                    config_path.clone(),
                                    new_config.wallpaper_dir.clone(),
                                    watch_tx.clone(),
                                ) {
                                    _watcher = w;
                                }
                            }

                            config = new_config;
                            included_media = get_included_media(&config);
                            current_index = 0;
                            next_tick = time::Instant::now() + Duration::from_secs(config.interval_seconds.max(1));
                        }
                    }
                    WatchEvent::DirEvent => {
                        if watcher::sync_media_items(&mut config).unwrap_or(false) {
                            let _ = config_mgr::save_config(&config).await;
                            included_media = get_included_media(&config);
                            current_index = 0;
                        }
                    }
                }
            }
        }
    }
}

