use anyhow::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tokio::task;

pub enum IpcCommand {
    Next,
}

pub fn get_socket_path() -> PathBuf {
    PathBuf::from("/tmp/smart-wallpaper.sock")
}

pub fn start_ipc_server(tx: mpsc::Sender<IpcCommand>) -> Result<()> {
    let socket_path = get_socket_path();

    // Clean up old socket
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(&socket_path)?;
    let _ = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o666));

    task::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let tx = tx.clone();
                task::spawn(async move {
                    let mut buf = [0; 64];
                    if let Ok(n) = stream.read(&mut buf).await {
                        if n > 0 {
                            let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                            if msg == "NEXT" {
                                let _ = tx.send(IpcCommand::Next).await;
                            }
                        }
                    }
                });
            }
        }
    });

    Ok(())
}
