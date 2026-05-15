# clash-tui

A cross-platform terminal UI for Clash/Mihomo proxy management, written in Rust.

![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-orange)
![Rust](https://img.shields.io/badge/rust-nightly-orange)

## Features

- **7 TUI Tabs** — Dashboard, Proxies, Connections, Mihomo Logs, Rules, Settings, Subscriptions
- **Embedded Mihomo Core** — v1.18.10 embedded at compile time, auto-starts in background
- **Non-blocking Startup** — TUI appears instantly, mihomo boots in background with real-time status
- **Subscription Management** — add/remove/subscribe with full TUI input (paste-safe, cursor navigation)
- **System Proxy** — one-key toggle Windows system proxy (Registry + WinINET notification)
- **Latency Testing** — concurrent per-proxy testing, color-coded results, sort by delay
- **3 Runtime Modes** — standalone (default), daemon (`--daemon`), client (`--port <N>`)
- **Themes** — Tokyo Night / Catppuccin / Gruvbox, `T` to cycle, saved to config
- **Vim Keybindings** — `j/k/↑↓` navigation, `Tab` page switch, `Esc` back

## Quick Start

```bash
# Download from releases, then run
.\clash-tui.exe
```

The embedded mihomo core starts automatically. Add a subscription URL in the **Subs** page (`a` to add, paste URL, `Enter` to confirm), then press `u` to download. Proxies appear in the **Proxies** page.

## Key Bindings

### Global
| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Next / Previous page |
| `q` / `Ctrl+C` | Quit |
| `?` | Help overlay |
| `r` | Force refresh |
| `T` | Cycle theme |

### Dashboard
| Key | Action |
|-----|--------|
| `m` | Cycle Clash mode (Rule→Global→Direct) |
| `1` / `2` / `3` | Set mode directly |
| `R` | Restart mihomo |

### Proxies
| Key | Action |
|-----|--------|
| `j/k` / `↑↓` | Navigate groups / proxies |
| `Enter` / `→` | Enter proxy list / Switch proxy |
| `Esc` / `←` | Back to group list |
| `t` | Test latency (single proxy in list, all in group) |
| `s` | Toggle sort by latency |

### Connections
| Key | Action |
|-----|--------|
| `j/k` / `↑↓` | Navigate connections |
| `d` | Close selected connection |
| `D` | Close all connections |

### Mihomo Logs
| Key | Action |
|-----|--------|
| `j/k` / `↑↓` | Scroll |
| `e` / `w` / `i` | Filter: Error / Warning / Info |
| `a` | Show all |
| `Home` / `End` | Jump top / bottom |
| `↑/k` disables auto-scroll, `End` re-enables |

### Settings
| Key | Action |
|-----|--------|
| `p` | Toggle system proxy |
| `P` | Enable system proxy |
| `o` | Disable system proxy |
| `T` | Cycle theme |

### Subscriptions
| Key | Action |
|-----|--------|
| `j/k` / `↑↓` | Navigate |
| `u` | Update / Download subscription |
| `e` | Toggle enable/disable |
| `a` | Add subscription (modal input) |
| `x` / `Delete` | Remove subscription |
| `Enter` | Confirm input |
| `Esc` | Cancel input |

## CLI Options

```
Usage: clash-tui.exe [OPTIONS]

Options:
  --daemon          Run as background daemon
  --install-core    Extract embedded mihomo to disk
  --port <PORT>     Connect to daemon (client mode)
  --host <HOST>     API host [default: 127.0.0.1]
  --api-port <PORT> API port [default: 9090]
```

## Build from Source

```bash
# Prerequisites: Rust nightly
git clone https://github.com/1879615351/clash-tui.git
cd clash-tui
cargo build --release
.\target\release\clash-tui.exe
```

The build script downloads mihomo v1.18.10 (~28MB) via `build.rs` and embeds it with `include_bytes!`. Final binary is ~35MB, fully self-contained.

### Linux

```bash
sudo apt install pkg-config libssl-dev build-essential
cargo build --release
./target/release/clash-tui
```

## Architecture

```
┌─────────────────────────────────────┐
│            clash-tui.exe             │
│  ┌──────────────────────────────┐   │
│  │     TUI (ratatui + crossterm) │   │
│  │  7 tabs, vim keybindings      │   │
│  ├──────────────────────────────┤   │
│  │     App Core                  │   │
│  │  event loop, state, dispatch  │   │
│  ├──────────────────────────────┤   │
│  │  ClashApi trait               │   │
│  │  ├─ HttpClashClient (direct)  │   │
│  │  └─ IpcClashClient  (daemon)  │── HTTP ──► mihomo :9090
│  ├──────────────────────────────┤   │
│  │  Startup Task (background)    │   │
│  │  polls /version → notifies UI │   │
│  └──────────────────────────────┘   │
│                                      │
│  mihomo v1.18.10 (embedded, 28MB)   │
│  auto-start, auto-cleanup            │
└─────────────────────────────────────┘
```

## Config

`%APPDATA%/clash-tui/config.toml` (Windows) or `~/.config/clash-tui/config.toml` (Linux):

```toml
[api]
host = "127.0.0.1"
port = 9090

[ui]
theme = "tokyo-night"       # tokyo-night | catppuccin | gruvbox
refresh_interval_ms = 1000

[core]
core_type = "mihomo"
core_path = ""              # empty = use embedded binary
```

### File locations

| File | Path |
|------|------|
| Config | `%APPDATA%/clash-tui/config.toml` |
| Subscriptions | `%APPDATA%/clash-tui/subscriptions.toml` |
| Mihomo core | `%APPDATA%/clash-tui/core/mihomo.exe` |
| Mihomo config | `%APPDATA%/clash-tui/core/config.yaml` |
| Mihomo log | `%APPDATA%/clash-tui/core/mihomo.log` |
| App log | `%APPDATA%/clash-tui/clash-tui.log` |

## Tech Stack

- **UI**: ratatui 0.29 + crossterm 0.28
- **Async**: tokio
- **HTTP**: reqwest 0.12
- **Serialization**: serde + serde_json + serde_yaml + toml
- **CLI**: clap 4
- **Embedded core**: build.rs → GitHub Releases download → `include_bytes!`
- **Platform (Win)**: winreg (system proxy), flate2 + zip (core extraction)

## License

MIT
