# clash-tui

A cross-platform terminal UI for Clash/Mihomo proxy management, written in Rust.

![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-orange)

## Features

- **7 TUI Pages** — Dashboard, Proxies, Connections, Logs, Rules, Settings, Subscriptions
- **Embedded Mihomo Core** — auto-start mihomo, no manual setup needed
- **Subscription Management** — add/update/remove Clash subscriptions with TUI input
- **System Proxy** — toggle Windows system proxy with one key
- **Latency Testing** — concurrent speed test for all proxies, auto-sort by delay
- **3 Runtime Modes** — standalone (default), daemon (`--daemon`), client (`--port`)
- **Themes** — Tokyo Night, Catppuccin, Gruvbox with runtime switching
- **Vim Keybindings** — `j/k/←/→/Tab` navigation throughout

## Screenshot

```
 Dashboard | Proxies | Conns | Logs | Rules | Settings | Subs
────────────────────────────────────────────────────────────────
┌ Core ───────────────┐ ┌ Mode ───────────────┐ ┌ Memory ────────────┐
│ Status: Running     │ │ Mode: RULE          │ │ ████████░░  132MB │
│ Version: v1.18.10   │ │                     │ └───────────────────┘
│ Conns: 24           │ │                     │
└─────────────────────┘ └─────────────────────┘
┌ Upload: 2.3MB/s ──────────────────────────────────────────────────┐
│ ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
└───────────────────────────────────────────────────────────────────┘
┌ Download: 10.8MB/s ───────────────────────────────────────────────┐
│ █████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
└───────────────────────────────────────────────────────────────────┘
  Disconnected | Mode: rule | Mem: 0MB | ↑ 0B ↓ 0B | Conns: 0
```

## Quick Start

```bash
# Download from releases, then run
.\clash-tui.exe
```

The embedded mihomo core starts automatically. Add a subscription URL in the Subs page, press `u` to download, and proxies appear in the Proxies page.

## Key Bindings

| Key | Action |
|-----|--------|
| `Tab` / `←→` | Switch page |
| `j/k` / `↑↓` | Navigate lists |
| `Enter` | Select proxy / Confirm |
| `t` | Test latency (single) / Test all in group |
| `s` | Toggle sort by latency |
| `m` | Cycle Clash mode (Rule→Global→Direct) |
| `p` | Toggle system proxy |
| `T` | Cycle theme |
| `d` / `D` | Close connection / Close all |
| `u` / `e` / `x` | Update / Toggle / Remove subscription |
| `a` | Add subscription (modal input) |
| `?` | Help overlay |
| `q` | Quit |

## Build from Source

```bash
# Prerequisites: Rust nightly, git
git clone https://github.com/1879615351/clash-tui.git
cd clash-tui

# Build (mihomo core downloaded automatically)
cargo build --release

# Run
.\target\release\clash-tui.exe
```

The build script downloads the mihomo core binary (~28MB) and embeds it via `include_bytes!`. The final executable is ~34MB and fully self-contained.

### Linux

```bash
sudo apt install pkg-config libssl-dev build-essential
cargo build --release
./target/release/clash-tui
```

## CLI Options

```
Usage: clash-tui.exe [OPTIONS]

Options:
  --daemon         Run as background daemon
  --install-core   Extract embedded mihomo to disk
  --port <PORT>    Connect to daemon (client mode)
  --host <HOST>    API host [default: 127.0.0.1]
  --api-port <PORT> API port [default: 9090]
```

## Architecture

```
┌─────────────────────┐     HTTP      ┌──────────────┐
│    clash-tui.exe     │ ◄──────────► │   mihomo     │
│   (TUI + API client) │   :9090      │   (proxy core)│
│                      │              │   :7890       │
│  ratatui + crossterm │              │   :7891       │
└─────────────────────┘              └──────────────┘
```

The embedded mihomo binary is compiled into the executable (28MB). On first run, it's extracted to `%APPDATA%/clash-tui/core/` and started automatically.

## Tech Stack

- **UI**: ratatui 0.29 + crossterm 0.28
- **Async**: tokio
- **HTTP**: reqwest
- **Serialization**: serde + serde_json + serde_yaml + toml
- **CLI**: clap 4
- **Platform**: windows-sys, winreg (Windows)

## Config

`%APPDATA%/clash-tui/config.toml` (Windows) or `~/.config/clash-tui/config.toml` (Linux):

```toml
[api]
host = "127.0.0.1"
port = 9090

[ui]
theme = "tokyo-night"
refresh_interval_ms = 1000

[core]
core_path = ""
core_type = "mihomo"
```

## License

MIT
