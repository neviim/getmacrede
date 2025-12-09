# Network Monitor Tool

A high-performance, secure CLI tool written in Rust for monitoring local networks. It provides real-time visibility into connected devices, their status, and notifies you of any changes.

## Features

### 🎨 NEW in v0.2.0: Visual Color System
- **Intuitive Color Coding**: Instantly identify device types with semantic colors
  - 🔵 Cyan IPs = VMs/Containers
  - 🟢 Green MACs = Physical hardware
  - 🟡 Yellow Virtual MACs = Virtual interfaces
  - 🟣 Magenta Vendors = Virtualization tech
- **Smart Deduplication**: Automatically eliminates duplicate IPs
- **Auto Virtual MAC Detection**: Detects and separates virtual MACs automatically

### Core Features

- **🚀 High Performance**: Built with Rust, `tokio` for async concurrency, and `pnet` for raw socket operations, ensuring fast and efficient scanning.
- **📊 Real-time Table View**: In monitor mode, displays a live-updating, color-coded table of all devices, sorted by IP.
- **🔔 Desktop Notifications**: Receive instant alerts when:
    - A **New Device** joins the network.
    - A device comes **Online**.
    - A device goes **Offline**.
- **💾 Persistence**: Automatically saves the list of known devices to `devices.json`, allowing state tracking across restarts.
- **🔍 Smart Detection**:
    - **Vendor Lookup**: Identifies 150+ manufacturers via OUI database
    - **Virtual MAC Detection**: Automatically detects VMs, containers, and virtual interfaces
    - **Hostname Resolution**: Multi-method (DHCP/DNS/mDNS/NetBIOS)
    - **Blacklist**: Blocks specific MAC addresses listed in `blacklist.json`
    - **MAC Mapping**: Manual correction for virtualized environments
    - **Status Tracking**: Intelligently marks devices as offline if they miss multiple scan intervals
- **🛡️ Secure**: Designed to run with minimal necessary privileges (requires `sudo` only for raw packet access).

## Prerequisites

- **Rust Toolchain**: Ensure you have Rust installed (`cargo` and `rustc`).
- **Root Privileges**: The tool requires `sudo` to send and receive raw ARP packets.

## Installation

Clone the repository and build the project in release mode for maximum performance:

```bash
cargo build --release
```

The binary will be located at `./target/release/getmacrede`.

## Usage

The tool has two main modes: `scan` and `monitor`.

### 1. Scan Mode
Performs a one-time scan of the specified network range and lists all detected devices.

```bash
sudo ./target/release/getmacrede scan --range <IP_RANGE> [OPTIONS]
```

**Example:**
```bash
sudo ./target/release/getmacrede scan --range 192.168.1.1-254
```

### 2. Monitor Mode
Continuously monitors the network, updating the status of devices in real-time and sending notifications.

```bash
sudo ./target/release/getmacrede monitor --range <IP_RANGE> [OPTIONS]
```

**Example:**
```bash
sudo ./target/release/getmacrede monitor --range 10.10.0.1-254 --interval 10
```

## Parameters

| Parameter | Flag | Description | Default | Required |
|-----------|------|-------------|---------|----------|
| **Range** | `-r`, `--range` | The IP range to scan (format: `START_IP-END_SUFFIX`). Example: `192.168.1.1-254` | N/A | Yes |
| **Interface** | `-i`, `--interface` | The network interface to use (e.g., `eth0`, `wlan0`). If omitted, it attempts to auto-detect. | Auto | No |
| **Interval** | `-n`, `--interval` | (Monitor mode only) The time in seconds between scans. | `30` | No |

## Documentation

- **[COLOR_GUIDE.md](COLOR_GUIDE.md)** - Complete visual color palette guide
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and detailed changes
- **[README_NOVAS_FUNCIONALIDADES.md](README_NOVAS_FUNCIONALIDADES.md)** - Feature documentation (Portuguese)

## Screenshots

### Monitor Mode with Color Coding
```
Network Monitor - Range: 192.168.15.1-254    Online: 30 | Offline: 10 | VMs: 16 | Total: 40
Last Scan: 01:20:33                                                        v0.2.0
Legend: IP: □ Physical | □ VM/Virtual | MAC: □ Physical | □ VM Real | □ VM Virtual
--------------------------------------------------------------------------------
IP              MAC               VIRTUAL MAC       HOSTNAME   STATUS   VENDOR
--------------------------------------------------------------------------------
192.168.15.1    24:2f:d0:7f:b6:e0                   gateway    Online   Intelbras
192.168.15.6                      bc:24:11:0e:b2:cb            Online   Proxmox VM
192.168.15.10   d0:94:66:a8:d8:72                              Online   Intel
```

**Color Indicators:**
- 🔵 Cyan IPs/MACs = Virtual devices
- 🟢 Green MACs = Physical hardware
- 🟡 Yellow Virtual MACs = VM interfaces
- 🟣 Magenta Vendors = Virtualization
- 🔴 Red Status = Offline/Blocked

## License
MIT
