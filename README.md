# System Monitor TUI

A cross-platform terminal UI system monitor built with Rust and Ratatui. Displays real-time system metrics in a compact, readable layout.

## Features

- **CPU**: Brand, core/thread count, frequency, total usage gauge, per-core bar charts
- **Memory**: RAM and swap usage with percentage gauges
- **GPU**: NVIDIA GPU monitoring via NVML (utilization, VRAM, temperature, power)
- **Network**: IPv4 interfaces with download/upload speeds, type detection (WiFi/ETH/VPN)
- **Storage**: SSD/HDD/NVME categorization with deduplicated physical drive sizes
- **Temperature**: CPU, GPU, SSD, battery sensors with color-coded thresholds
- **Processes**: Top processes sorted by CPU and memory usage

## Tech Stack

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | 2024 edition | Language |
| Ratatui | 0.30.0 | TUI framework |
| Crossterm | 0.29.0 | Terminal manipulation |
| Sysinfo | 0.33.0 | System information |
| NVML-Wrapper | 0.10.0 | NVIDIA GPU monitoring |

## Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Without NVIDIA support (faster compile, smaller binary)
cargo build --release --no-default-features
```

## Usage

```bash
./target/release/ratatui-counter-app
```

| Key | Action |
|-----|--------|
| `q` | Quit |

## Layout

```
─────────────────────────────────────────────────────────┐
│                    System Monitor                       │
──────────────┬──────────────┬──────────────┬────────────┤
│     CPU      │   Memory     │     GPU      │  Network   │
├──────────────┼──────────────┼──────────────┼────────────┤
│   Storage    │    Temp      │  Proc (CPU)  │ Proc (MEM) │
├──────────────┴──────────────┴──────────────────────────┤
│                      Quit <Q>                           │
└─────────────────────────────────────────────────────────┘
```

## Cross-Platform

| Platform | Support | Notes |
|----------|---------|-------|
| macOS | Full | Apple Silicon aggregate CPU, NVML optional |
| Linux | Full | Per-core CPU, NVML required for GPU |
| Windows | Full | Per-core CPU, NVML required for GPU |

### Platform-Specific Behavior

**CPU Cores**: macOS Apple Silicon exposes only 1 aggregate CPU entry via sysinfo. The widget displays "ALL" instead of individual cores. Linux and Windows show per-core bars.

**GPU**: NVIDIA GPUs detected via NVML on Linux/Windows. macOS shows "No NVIDIA GPU" (Apple Silicon GPUs not supported by NVML).

**Storage Deduplication**:
- macOS: `disk0s1` + `disk0s2` → single `disk0` entry
- Linux: `sda1` + `sda2` → single `sda`, `nvme0n1p1` → `nvme0n1`
- Windows: Drive letters (`C:`, `D:`) shown separately

**Network Interface Detection**:
- macOS: `en0` → WiFi, `utun*` → VPN
- Linux: `wlan*` → WiFi, `eth*`/`eno*`/`ens*` → ETH, `wg*` → VPN
- Windows: Interface prefix fallback

**Temperature Sensors**:
- macOS: Filters PMU/tdev/ANS noise, shows CPU/GPU/SSD/BAT
- Linux: `acpitz` → SYS, `pch` → PCH, `tctl` → CPU
- Windows: Generic fallback

## License

MIT
