
#  SmartPaper

**An intelligent wallpaper manager for Wayland Linux desktops.**

Automatically rotates your wallpapers on a configurable timer, supports both static images and live video wallpapers, and comes with a sleek Tauri-based GUI for managing everything.

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri_v2-FFC131?style=for-the-badge&logo=tauri&logoColor=black)](https://v2.tauri.app/)
[![React](https://img.shields.io/badge/React_19-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://react.dev/)
[![Wayland](https://img.shields.io/badge/Wayland-FFB81C?style=for-the-badge&logo=wayland&logoColor=black)](https://wayland.freedesktop.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)

</div>

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Screenshots](#screenshots)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Arch Linux / Manjaro](#arch-linux--manjaro)
  - [Fedora / RHEL](#fedora--rhel)
  - [Ubuntu / Debian](#ubuntu--debian)
  - [openSUSE](#opensuse)
  - [Void Linux](#void-linux)
  - [NixOS](#nixos)
- [Building from Source](#building-from-source)
- [Configuration](#configuration)
- [Usage](#usage)
- [Systemd Service](#systemd-service)
- [How It Works](#how-it-works)
- [Supported Formats](#supported-formats)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

SmartPaper is a two-component wallpaper engine built in Rust for Wayland-based Linux desktops:

1. **Daemon** — A lightweight background service that handles wallpaper rotation, file system monitoring, and IPC.
2. **App** — A Tauri v2 desktop application (React + TypeScript frontend) that provides a visual interface to manage your wallpaper collection, toggle individual wallpapers on/off, adjust rotation intervals, and skip to the next wallpaper.

The daemon watches your wallpaper directory for changes in real-time, automatically discovers new images and videos, and rotates through them on a configurable interval. The GUI communicates with the daemon over a Unix domain socket for instant control.

---

## Features

-  **Automatic Rotation** — Cycle through wallpapers on a configurable timer (default: 5 minutes)
-  **Video Wallpapers** — Full support for live/animated wallpapers using `mpvpaper`
-  **Directory Watching** — Real-time file system monitoring via `notify`; new wallpapers are picked up automatically
-  **Tauri GUI** — Modern, dark-themed desktop app to browse thumbnails, toggle wallpapers, and control playback
-  **Shuffle / Sequential** — Switch between random and sequential rotation order
-  **Skip to Next** — Instantly jump to the next wallpaper via GUI button, IPC command, or global hotkey (`Super + Alt + N`)
-  **Thumbnail Generation** — Automatic video thumbnail generation using `ffmpeg`
-  **Lightweight** — Async Rust daemon with minimal resource usage (tokio runtime)
-  **IPC Control** — Unix socket interface at `/tmp/smart-wallpaper.sock` for scripting and external control
-  **Live Config Reload** — Edit `config.json` externally and the daemon picks up changes instantly
-  **Multi-Monitor** — Video wallpapers span all outputs via `mpvpaper '*'`

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        SmartPaper                               │
├─────────────────────┬──────────────────────┬────────────────────┤
│     shared/         │      daemon/         │       app/         │
│  (Rust Library)     │   (Rust Binary)      │  (Tauri v2 App)    │
│                     │                      │                    │
│  • Config struct    │  • Wallpaper Manager │  • React 19 UI     │
│  • MediaItem model  │  • File Watcher      │  • Tailwind CSS    │
│  • MediaType enum   │  • Config Manager    │  • Tauri Commands  │
│                     │  • IPC Server        │  • Thumbnail Gen   │
│                     │  • Rotation Engine   │  • Directory Picker│
└─────────────────────┴──────────────────────┴────────────────────┘
                              │                        │
                    Unix Socket IPC            Reads/Writes
                   /tmp/smart-wallpaper.sock    config.json
                              │                        │
                              └────────────────────────┘
```

The project is a **Cargo workspace** with three crates:

| Crate | Type | Description |
|-------|------|-------------|
| `shared` | Library | Shared data models (`Config`, `MediaItem`, `MediaType`) used by both daemon and app |
| `daemon` | Binary | Background service that rotates wallpapers, watches files, and listens for IPC commands |
| `app/src-tauri` | Tauri App | Desktop GUI built with React 19 + TypeScript, bundled via Tauri v2 |

---

## Prerequisites

The following tools and libraries are required regardless of distribution:

| Dependency | Purpose | Required By |
|---|---|---|
| **Rust toolchain** (≥ 1.75) | Building the Rust crates | All |
| **Node.js** (≥ 18) + **npm** | Building the frontend | App |
| **Tauri v2 CLI** | Building and bundling the desktop app | App |
| **mpvpaper** | Rendering video wallpapers on Wayland | Daemon (video) |
| **ffmpeg** | Generating video thumbnails | App |
| **webkit2gtk 4.1** | Tauri WebView rendering | App |
| **A Wayland compositor** | Display server (Hyprland, Sway, etc.) | All |

---

## Installation

### Arch Linux / Manjaro

```bash
# Install system dependencies
sudo pacman -S --needed rust nodejs npm webkit2gtk-4.1 ffmpeg mpvpaper \
    base-devel openssl pkgconf gtk3 libayatana-appindicator librsvg

# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Clone and build
git clone https://github.com/Nexusdeveloper902/SmartPaper.git
cd SmartPaper
cargo build --release

# Build the Tauri app
cd app
npm install
cargo tauri build
cd ..
```

### Fedora / RHEL

```bash
# Install system dependencies
sudo dnf install rust cargo nodejs npm webkit2gtk4.1-devel ffmpeg mpvpaper \
    openssl-devel pkgconf-pkg-config gtk3-devel libayatana-appindicator-gtk3-devel \
    librsvg2-devel gcc-c++

# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Clone and build
git clone https://github.com/Nexusdeveloper902/SmartPaper.git
cd SmartPaper
cargo build --release

# Build the Tauri app
cd app
npm install
cargo tauri build
cd ..
```

### Ubuntu / Debian

```bash
# Install system dependencies
sudo apt update
sudo apt install -y build-essential curl wget file libssl-dev libgtk-3-dev \
    libayatana-appindicator3-dev librsvg2-dev libwebkit2gtk-4.1-dev \
    ffmpeg nodejs npm

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install mpvpaper (may need to build from source)
# Check your distro repos or build from: https://github.com/GhostNaN/mpvpaper
sudo apt install -y mpvpaper || echo "mpvpaper may need to be built from source"

# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Clone and build
git clone https://github.com/Nexusdeveloper902/SmartPaper.git
cd SmartPaper
cargo build --release

# Build the Tauri app
cd app
npm install
cargo tauri build
cd ..
```

### openSUSE

```bash
# Install system dependencies
sudo zypper install rust cargo nodejs npm webkit2gtk3-devel ffmpeg \
    libopenssl-devel pkg-config gtk3-devel libayatana-appindicator3-devel \
    librsvg-devel gcc-c++ patterns-devel-base-devel_basis

# Install mpvpaper (check OBS repos or build from source)
# https://github.com/GhostNaN/mpvpaper

# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Clone and build
git clone https://github.com/Nexusdeveloper902/SmartPaper.git
cd SmartPaper
cargo build --release

cd app
npm install
cargo tauri build
cd ..
```

### Void Linux

```bash
# Install system dependencies
sudo xbps-install -S rust cargo nodejs base-devel webkit2gtk-devel \
    ffmpeg openssl-devel pkg-config gtk+3-devel librsvg-devel

# Install mpvpaper
sudo xbps-install -S mpvpaper || echo "Build from source if not available"

# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Clone and build
git clone https://github.com/Nexusdeveloper902/SmartPaper.git
cd SmartPaper
cargo build --release

cd app
npm install
cargo tauri build
cd ..
```

### NixOS

Add the following to a `shell.nix` or use `nix-shell`:

```nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustc cargo pkg-config nodejs
    tauri-cli
  ];
  buildInputs = with pkgs; [
    openssl webkitgtk_4_1 gtk3
    libayatana-appindicator librsvg
    ffmpeg mpvpaper
  ];
}
```

Then:
```bash
nix-shell
git clone https://github.com/Nexusdeveloper902/SmartPaper.git
cd SmartPaper
cargo build --release
cd app && npm install && cargo tauri build
```

---

## Building from Source

```bash
# Clone the repository
git clone https://github.com/Nexusdeveloper902/SmartPaper.git
cd SmartPaper

# Build the daemon (release mode)
cargo build --release --bin daemon

# Build the Tauri app
cd app
npm install
cargo tauri build

# Binaries will be at:
#   Daemon:  ./target/release/daemon
#   App:     ./app/src-tauri/target/release/app
```

---

## Configuration

SmartPaper stores its configuration at:

```
~/.config/com.smart-wallpaper.app/config.json
```

### Config Schema

```json
{
  "wallpaper_dir": "/home/user/Pictures/Wallpapers",
  "interval_seconds": 300,
  "random": true,
  "media": [
    {
      "path": "/home/user/Pictures/Wallpapers/mountain.jpg",
      "media_type": "image",
      "included": true
    },
    {
      "path": "/home/user/Pictures/Wallpapers/rain.mp4",
      "media_type": "video",
      "included": true
    }
  ]
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `wallpaper_dir` | `string \| null` | `null` | Path to the directory containing wallpaper files |
| `interval_seconds` | `integer` | `300` | Rotation interval in seconds (minimum: 1) |
| `random` | `boolean` | `true` | Shuffle wallpapers randomly instead of alphabetical order |
| `media` | `array` | `[]` | Auto-discovered media items with include/exclude toggles |

The `media` array is **automatically managed** — the daemon scans `wallpaper_dir` and syncs entries. You can toggle `included` to `false` via the GUI to skip specific wallpapers.

---

## Usage

### Running the Daemon

```bash
# Start the daemon directly
./target/release/daemon

# Or install and run via systemd (see below)
```

### Running the GUI App

```bash
# Run the Tauri app
cd app
cargo tauri dev

# Or run the built binary
./app/src-tauri/target/release/app
```

### IPC Commands

You can control the daemon externally via the Unix socket:

```bash
# Skip to next wallpaper
echo -n "NEXT" | socat - UNIX-CONNECT:/tmp/smart-wallpaper.sock
```

---

## Systemd Service

A systemd user service file is included for running the daemon as a background service:

```bash
# Copy the service file
cp smart-wallpaper.service ~/.config/systemd/user/

# Edit the ExecStart path to match your installation
# nano ~/.config/systemd/user/smart-wallpaper.service

# Enable and start
systemctl --user daemon-reload
systemctl --user enable smart-wallpaper.service
systemctl --user start smart-wallpaper.service

# Check status
systemctl --user status smart-wallpaper.service

# View logs
journalctl --user -u smart-wallpaper.service -f
```

> **Note:** Make sure to update the `ExecStart` path in the service file to point to your built `daemon` binary.

---

## How It Works

1. **Startup** — The daemon loads `config.json`, scans the wallpaper directory, and sets the initial wallpaper.
2. **File Watching** — `notify` watches both the config file and the wallpaper directory for changes.
3. **Rotation Loop** — A `tokio::select!` loop handles three event sources concurrently:
   - **Timer tick** — Advances to the next wallpaper when the interval expires
   - **IPC command** — Handles `NEXT` commands from the GUI or external scripts
   - **File events** — Reloads config or re-scans the directory when changes are detected
4. **Wallpaper Setting**:
   - **Images** — Set via a configurable shell script (e.g., `swww`, `swaybg`, or custom scripts)
   - **Videos** — Rendered as live wallpapers using `mpvpaper` with infinite loop
5. **GUI** — The Tauri app reads/writes the same `config.json` and sends IPC commands to the daemon over the Unix socket.

---

## Supported Formats

### Images
| Format | Extension |
|--------|-----------|
| JPEG | `.jpg`, `.jpeg` |
| PNG | `.png` |
| WebP | `.webp` |

### Videos
| Format | Extension |
|--------|-----------|
| MP4 | `.mp4` |
| WebM | `.webm` |
| Matroska | `.mkv` |

---

## Troubleshooting

### Daemon won't set wallpapers
- Ensure you're running on a **Wayland** session
- Check that `mpvpaper` is installed and in your `$PATH` (for video wallpapers)
- Verify the wallpaper directory path in `config.json` is correct

### GUI can't connect to daemon
- Make sure the daemon is running (`systemctl --user status smart-wallpaper.service`)
- Check that `/tmp/smart-wallpaper.sock` exists
- Ensure no permission issues on the socket file

### Video wallpapers not working
- Install `mpvpaper`: https://github.com/GhostNaN/mpvpaper
- Ensure your compositor supports layer-shell protocol

### Thumbnails not generating
- Install `ffmpeg` and ensure it's in your `$PATH`

### App crashes on launch from desktop shortcut
- Common stability issues on Wayland/GTK (such as `SIGSEGV` in `libatk-bridge` or WebKit rendering glitches) are now mitigated by internal environment overrides:
  - `WEBKIT_DISABLE_DMABUF_RENDERER=1`
  - `NO_AT_BRIDGE=1`
  - `GTK_A11Y=none`
- If issues persist, ensure Wayland environment variables are exported (e.g., `GDK_BACKEND=wayland`).
- Use `systemd-run --user --scope` to isolate the process from the launcher if necessary.

---

## Contributing

Contributions are welcome! Feel free to open issues and pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <sub>Built with ❤️ in Rust</sub>
</div>
