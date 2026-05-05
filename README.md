# sysmon-tui

> System monitor TUI — real-time resource monitoring in your terminal.

A lightweight, async system monitor written in Rust that provides a comprehensive dashboard for tracking CPU, memory, processes, disks, network, and Docker containers — all updated every second in a rich terminal interface.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)

## Features

| Tab | Description |
|-----|-------------|
| **CPU** | Overall usage gauge, per-core monitoring, 60s sparkline history, CPU model/frequency, load average |
| **Memory** | RAM and Swap gauges with color-coded usage, detailed breakdown (used, available, cached, buffers) |
| **Processes** | Sortable table with PID, name, CPU%, MEM%, user, command — scrollable with page navigation |
| **Disks** | Per-disk table (mount, FS, total, used, usage%) with fill gauges |
| **Network** | Per-interface stats (MAC, IPv4, throughput), RX/TX sparklines, download/upload speed display |
| **Docker** | All containers (running, stopped, created) with state colors, images table, connection error handling |

## Architecture

```
┌─────────────┐   mpsc channel   ┌─────────────┐
│  Collector  │ ───────────────▶ │     App     │
│  (tokio)    │  SystemMetrics   │   (UI loop) │
└─────────────┘                  └─────────────┘
```

- **Async collector task** (`tokio::spawn`): refreshes system metrics every interval and sends them via `mpsc::Sender`
- **UI task** (main): renders `ratatui` widgets, listens for `crossterm` events and new metrics via `mpsc::Receiver`
- `sysinfo` for system metrics, `docker-api` for Docker socket communication

## Installation

### Prerequisites

- **Rust 1.75+** (edition 2024)
- **Linux** (primary target)
- Access to `/var/run/docker.sock` for Docker tab (optional)

### From source

```bash
git clone https://github.com/ninila-company/sysmon-tui.git
cd sysmon-tui
cargo build --release
```

The binary will be at `target/release/sysmon-tui`.

### Install locally

```bash
cargo install --path .
```

## Usage

```bash
# Run with default settings (1000ms refresh rate)
sysmon-tui

# Custom refresh rate (milliseconds)
sysmon-tui -r 500

# Show help
sysmon-tui --help
```

### CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `-r, --refresh-rate <ms>` | Metrics refresh interval in milliseconds | `1000` |
| `-h, --help` | Show help message | — |
| `-V, --version` | Show version | — |

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `1` | Switch to CPU tab |
| `2` | Switch to Memory tab |
| `3` | Switch to Processes tab |
| `4` | Switch to Disks tab |
| `5` | Switch to Network tab |
| `6` | Switch to Docker tab |
| `Tab` | Cycle through tabs |
| `q` / `Esc` | Quit application |

### Process list (Tab 3)

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate processes |
| `PgUp` / `PgDn` | Page scroll |
| `s` | Cycle sort column (PID → Name → CPU% → MEM%) |
| `s` (repeated) | Toggle sort direction (▼ / ▲) |

### General

| Key | Action |
|-----|--------|
| `?` | Toggle help overlay |

## Screenshots

### CPU Tab
```
┌─ hostname@192.168.1.50 | Ubuntu 24.04 ──────────┬─ Memory ───────────────────┬─ sysmon-tui ──────┐
│ CPU: 23.4% | 12 logical cores                   │ RAM: 4.2 / 16.0 GB         │ v0.1.0 | 1000ms │
└──────────────────────────────────────────────────┴────────────────────────────┴──────────────────┘
┌─ CPU ───────────────────────────────────────────────────────────────────────────────────────────┐
│ Overall CPU: [████████░░░░░░░░░░] 23.4% | 12 logical cores                                      │
│ Per-Core: Core  0: 24.1% │ Core  1: 18.3% │ ...                                                  │
│ CPU History (60s): ▁▂▃▅▇█▇▅▃▂▁▁▂▃▄▅▆▅▄▃▂▁▁▂▃▄▅▆▇█▇▅▄▃▂▁                                        │
│ CPU Info: Model: AMD Ryzen 7 | Freq: 3.60 GHz | Load Avg: 1.23 0.89 0.76                         │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
┌─ 1:CPU ─ 2:MEM ─ 3:PROC ─ 4:DISK ─ 5:NET ─ 6:DOCKER │ ↑↓:scroll │ s:sort │ ?:help │ q:quit ─┐│ Up: 1d 05h 23m │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.29 | Terminal UI framework |
| `crossterm` | 0.28 | Terminal backend (raw mode, events) |
| `sysinfo` | 0.33 | System metrics (CPU, memory, processes, disks, network) |
| `clap` | 4 | CLI argument parsing |
| `tokio` | 1 | Async runtime |
| `docker-api` | 0.12 | Docker socket communication |
| `chrono` | 0.4 | Timestamp formatting |
| `anyhow` | 1 | Error handling |

## Development

```bash
# Debug build
cargo build

# Run
cargo run

# Release build
cargo build --release
```

## Project structure

```
sysmon-tui/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, tokio runtime, collector task, event loop
│   ├── app.rs               # Application state (App struct, enums, data models)
│   ├── events.rs            # Keyboard event handling
│   └── ui/
│       ├── mod.rs           # Layout orchestration, header, footer
│       ├── cpu.rs           # CPU tab: gauge, per-core bars, sparkline, info
│       ├── memory.rs        # Memory tab: RAM/Swap gauges, detailed breakdown
│       ├── processes.rs     # Processes tab: sortable table
│       ├── disks.rs         # Disks tab: table + fill gauges
│       ├── network.rs       # Network tab: throughput, sparklines, interface table
│       └── docker.rs        # Docker tab: containers + images tables
```

## License

MIT
