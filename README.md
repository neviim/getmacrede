# Network Monitor Tool

A high-performance, secure CLI tool written in Rust for monitoring local networks. It provides real-time visibility into connected devices, their status, and notifies you of any changes.

## Features

- **🚀 High Performance**: Built with Rust, `tokio` for async concurrency, and `pnet` for raw socket operations, ensuring fast and efficient scanning.
- **📊 Real-time Table View**: In monitor mode, displays a live-updating table of all devices, sorted by IP, showing their current status (Online/Offline).
- **🔔 Desktop Notifications**: Receive instant alerts when:
    - A **New Device** joins the network.
    - A device comes **Online**.
    - A device goes **Offline**.
- **💾 Persistence**: Automatically saves the list of known devices to `devices.json`, allowing state tracking across restarts.
- **🔍 Smart Detection**:
    - **Hostname Resolution**: Resolves DNS/mDNS names to identify devices easily.
    - **Blacklist**: Blocks specific MAC addresses listed in `blacklist.json`, displaying them in red with "Block" status.
    - **Version Display**: Shows the current application version on startup.
    - **Status Tracking**: Intelligently marks devices as offline if they miss multiple scan intervals.
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

## License
MIT
