use chrono::Utc;
use colored::*;
use notify_rust::Notification;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::time::Duration;
use tokio::time;

use crate::models::{Device, DeviceStatus};
use crate::scanner;
use crate::utils;

const STORAGE_FILE: &str = "devices.json";
const BLACKLIST_FILE: &str = "blacklist.json";

pub async fn run_monitor(interface: Option<String>, range: String, interval: u64) {
    let target_ips = match utils::parse_ip_range(&range) {
        Ok(ips) => ips,
        Err(e) => {
            eprintln!("Error parsing IP range: {}", e);
            return;
        }
    };

    println!(
        "{}",
        format!("Starting monitor on range: {}", range)
            .green()
            .bold()
    );
    println!("Press Ctrl+C to stop.");

    let known_devices = load_devices().unwrap_or_default();
    let mut device_map: HashMap<String, Device> = known_devices
        .into_iter()
        .map(|d| (d.mac.clone(), d))
        .collect();

    let mut interval_timer = time::interval(Duration::from_secs(interval));

    loop {
        interval_timer.tick().await;

        // Load Blacklist
        let blacklist = load_blacklist().unwrap_or_default();

        let found_devices = scanner::scan_network(interface.clone(), target_ips.clone()).await;
        let mut changes = false;

        // 1. Process Found Devices
        for found in found_devices {
            let is_blocked = blacklist.contains(&found.mac);

            if let Some(existing) = device_map.get_mut(&found.mac) {
                // Update existing
                if existing.status == DeviceStatus::Offline {
                    notify("Device Online", &format!("{} is back online", found.ip));
                    existing.status = if is_blocked {
                        DeviceStatus::Block
                    } else {
                        DeviceStatus::Online
                    };
                    changes = true;
                } else if is_blocked && existing.status != DeviceStatus::Block {
                    existing.status = DeviceStatus::Block;
                    changes = true;
                } else if !is_blocked && existing.status == DeviceStatus::Block {
                    existing.status = DeviceStatus::Online;
                    changes = true;
                }

                // Update metadata if changed
                if existing.ip != found.ip {
                    existing.ip = found.ip.clone();
                    changes = true;
                }
                if found.hostname.is_some() && existing.hostname != found.hostname {
                    existing.hostname = found.hostname.clone();
                    changes = true;
                }
                existing.last_seen = Utc::now();
            } else {
                // New Device
                notify(
                    "New Device Detected",
                    &format!("IP: {}\nMAC: {}", found.ip, found.mac),
                );

                let mut new_device = found.clone();
                if is_blocked {
                    new_device.status = DeviceStatus::Block;
                }
                device_map.insert(new_device.mac.clone(), new_device);
                changes = true;
            }
        }

        // 2. Check for Offline Devices
        let now = Utc::now();
        for device in device_map.values_mut() {
            let is_blocked = blacklist.contains(&device.mac);

            // If blocked, ensure status is Block
            if is_blocked {
                if device.status != DeviceStatus::Block {
                    device.status = DeviceStatus::Block;
                    changes = true;
                }
                continue; // Skip offline check for blocked devices
            }

            if device.status == DeviceStatus::Online {
                let threshold =
                    chrono::Duration::seconds((interval as i64 * 2) + (interval as i64 / 2));
                if now.signed_duration_since(device.last_seen) > threshold {
                    notify("Device Offline", &format!("{} went offline", device.ip));
                    device.status = DeviceStatus::Offline;
                    changes = true;
                }
            }
        }

        if changes {
            let devices: Vec<Device> = device_map.values().cloned().collect();
            if let Err(e) = save_devices(&devices) {
                eprintln!("Failed to save devices: {}", e);
            }
        }

        // 3. Display Table
        // Clear screen and move to top
        print!("\x1B[2J\x1B[1;1H");

        println!(
            "{} v{}",
            format!("Network Monitor - Range: {}", range).green().bold(),
            env!("CARGO_PKG_VERSION")
        );
        println!("Last Scan: {}", Utc::now().format("%H:%M:%S"));
        println!("{}", "-".repeat(65));
        println!(
            "{:<15} {:<17} {:<20} {:<10}",
            "IP", "MAC", "HOSTNAME", "STATUS"
        );
        println!("{}", "-".repeat(65));

        let mut devices: Vec<&Device> = device_map.values().collect();
        // Sort by IP
        devices.sort_by(|a, b| {
            let ip_a =
                a.ip.parse::<std::net::Ipv4Addr>()
                    .unwrap_or(std::net::Ipv4Addr::new(0, 0, 0, 0));
            let ip_b =
                b.ip.parse::<std::net::Ipv4Addr>()
                    .unwrap_or(std::net::Ipv4Addr::new(0, 0, 0, 0));
            ip_a.cmp(&ip_b)
        });

        for device in devices {
            let hostname = device.hostname.as_deref().unwrap_or("-");
            let status_str = device.status.to_string();
            let status_colored = match device.status {
                DeviceStatus::Online => status_str.green(),
                DeviceStatus::Offline => status_str.red(),
                DeviceStatus::Block => status_str.red().bold(),
            };

            println!(
                "{:<15} {:<17} {:<20} {:<10}",
                device.ip,
                device.mac,
                hostname.chars().take(20).collect::<String>(),
                status_colored
            );
        }
    }
}

fn load_devices() -> io::Result<Vec<Device>> {
    if !Path::new(STORAGE_FILE).exists() {
        return Ok(Vec::new());
    }
    let file = File::open(STORAGE_FILE)?;
    let reader = BufReader::new(file);
    let devices = serde_json::from_reader(reader)?;
    Ok(devices)
}

fn load_blacklist() -> io::Result<Vec<String>> {
    if !Path::new(BLACKLIST_FILE).exists() {
        return Ok(Vec::new());
    }
    let file = File::open(BLACKLIST_FILE)?;
    let reader = BufReader::new(file);
    let blacklist = serde_json::from_reader(reader)?;
    Ok(blacklist)
}

fn save_devices(devices: &[Device]) -> io::Result<()> {
    let file = File::create(STORAGE_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, devices)?;
    Ok(())
}

fn notify(summary: &str, body: &str) {
    let _ = Notification::new().summary(summary).body(body).show();
}
