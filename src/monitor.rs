use chrono::Utc;
use colored::*;
use fs2::FileExt;
use notify_rust::Notification;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::time::Duration;
use tokio::time;

use crate::models::{Device, DeviceStatus};
use crate::proxmox;
use crate::scanner;
use crate::utils;
use crate::vendor::VendorDb;

/// Helper function to pad a colored string to a specific width
/// ANSI color codes don't count toward visible width, so we need custom padding
fn pad_colored(s: String, width: usize) -> String {
    // If string is empty, return spaces
    if s.is_empty() {
        return " ".repeat(width);
    }

    // Count visible characters (excluding ANSI codes)
    let visible_len = console::strip_ansi_codes(&s).len();
    let padding = if visible_len < width {
        width - visible_len
    } else {
        0
    };
    format!("{}{}", s, " ".repeat(padding))
}

const STORAGE_FILE: &str = "devices.json";
const BLACKLIST_FILE: &str = "blacklist.json";

pub async fn run_monitor(interface: Option<String>, range: String, interval: u64, resolve_hostnames: bool) {
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

    // Check if MAC mapping file exists
    if Path::new("mac_mapping.json").exists() {
        println!("{}", "MAC address mapping enabled".cyan());
    } else {
        println!("{}", "MAC address mapping disabled".yellow());
        println!("Create mac_mapping.json to correct MAC addresses for virtualized devices");
    }

    println!("Press Ctrl+C to stop.");

    let known_devices = load_devices().unwrap_or_default();

    // Load manual MAC mappings (IP -> Real MAC) once at startup
    let mac_mappings = proxmox::load_mac_mappings().unwrap_or_default();

    // Create vendor database for lookup
    let vendor_db = VendorDb::new();

    // Apply MAC mappings and vendor lookup to existing devices
    let mut mapping_applied = false;
    let mut device_map: HashMap<String, Device> = HashMap::new();

    for mut d in known_devices {
        // Apply MAC mapping if exists
        if let Some(real_mac) = mac_mappings.get(&d.ip) {
            if &d.mac != real_mac {
                if d.virtual_mac.is_none() {
                    d.virtual_mac = Some(d.mac.clone());
                }
                d.mac = real_mac.clone();
                mapping_applied = true;
            }
        } else if d.virtual_mac.is_none() && vendor_db.is_virtual(&d.mac) {
            // Auto-detect virtual MAC for existing devices without manual mapping
            d.virtual_mac = Some(d.mac.clone());
            mapping_applied = true;
        }

        // Apply vendor lookup if not present
        if d.vendor.is_none() {
            d.vendor = vendor_db.lookup(&d.mac);
            mapping_applied = true;
        }

        // Validate device before inserting into map
        if !d.validate() {
            eprintln!("WARNING: Skipping invalid device during initialization: IP='{}', MAC='{}'", d.ip, d.mac);
            continue;
        }

        // Use IP as key instead of MAC to prevent duplicates
        // If there's already a device with this IP, keep the most recently seen one
        if let Some(existing) = device_map.get(&d.ip) {
            if d.last_seen > existing.last_seen {
                device_map.insert(d.ip.clone(), d);
                mapping_applied = true;
            }
        } else {
            device_map.insert(d.ip.clone(), d);
        }
    }

    // Save devices if mappings were applied
    if mapping_applied {
        let devices: Vec<Device> = device_map.values().cloned().collect();
        if let Err(e) = save_devices(&devices) {
            eprintln!("Failed to save devices after applying MAC mappings: {}", e);
        }
    }

    let mut interval_timer = time::interval(Duration::from_secs(interval));

    loop {
        interval_timer.tick().await;

        // Flush ARP cache before scanning for fresh MAC addresses
        if let Err(e) = utils::flush_arp_cache(interface.as_deref()) {
            eprintln!("Warning: Failed to flush ARP cache: {}", e);
            eprintln!("Note: Flushing ARP cache requires root/sudo privileges");
        }

        // Load Blacklist
        let blacklist = load_blacklist().unwrap_or_default();

        let mut found_devices = scanner::scan_network(interface.clone(), target_ips.clone(), resolve_hostnames).await;

        // Correct MAC addresses using manual mappings and auto-detect virtual MACs
        for device in &mut found_devices {
            if let Some(real_mac) = mac_mappings.get(&device.ip) {
                // Store the virtual MAC and replace with real MAC
                device.virtual_mac = Some(device.mac.clone());
                device.mac = real_mac.clone();
                // Update vendor for the real MAC
                device.vendor = vendor_db.lookup(&device.mac);
            } else if vendor_db.is_virtual(&device.mac) {
                // Auto-detect virtual MAC without manual mapping
                // Move the virtual MAC to virtual_mac field and use a placeholder for mac
                device.virtual_mac = Some(device.mac.clone());
                // Keep the virtual MAC in the mac field but mark it clearly in vendor
                // This ensures the device has a valid MAC identifier
            }
        }

        let mut changes = false;

        // 1. Process Found Devices
        for found in found_devices {
            // Validate found device before processing
            if !found.validate() {
                eprintln!("WARNING: Scanner returned invalid device, skipping: IP='{}', MAC='{}'", found.ip, found.mac);
                continue;
            }

            let is_blocked = blacklist.contains(&found.mac);

            if let Some(existing) = device_map.get_mut(&found.ip) {
                // Update existing device
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

                // Update MAC if changed (for devices with dynamic MACs)
                if existing.mac != found.mac {
                    // If the new MAC is virtual, move it to virtual_mac
                    if found.virtual_mac.is_some() {
                        existing.virtual_mac = found.virtual_mac.clone();
                        // Don't update the mac field if it's already a real MAC
                        if existing.virtual_mac.as_ref() != Some(&existing.mac) {
                            // Keep the real MAC if we have one
                        } else {
                            // Update if both are virtual
                            existing.mac = found.mac.clone();
                        }
                    } else {
                        // New MAC is not virtual, update it
                        existing.mac = found.mac.clone();
                    }
                    changes = true;
                }

                // Update virtual_mac if changed
                if found.virtual_mac.is_some() && existing.virtual_mac != found.virtual_mac {
                    existing.virtual_mac = found.virtual_mac.clone();
                    changes = true;
                }

                // Update vendor if changed
                if found.vendor.is_some() && existing.vendor != found.vendor {
                    existing.vendor = found.vendor.clone();
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
                device_map.insert(new_device.ip.clone(), new_device);
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

        // Calculate statistics first (moved up to use in header)
        let mut devices: Vec<&Device> = device_map.values().collect();
        let total = devices.len();
        let online = devices.iter().filter(|d| d.status == DeviceStatus::Online).count();
        let offline = devices.iter().filter(|d| d.status == DeviceStatus::Offline).count();
        let vms = devices.iter().filter(|d| {
            if let Some(vendor) = &d.vendor {
                vendor.contains("Virtual") || vendor.contains("Proxmox") ||
                vendor.contains("QEMU") || vendor.contains("VMware") ||
                vendor.contains("Hyper-V") || vendor.contains("VirtualBox")
            } else {
                false
            }
        }).count();

        // Create title with stats aligned to the right
        let title_str = format!("Network Monitor - Range: {}", range);
        let stats_str = format!("Online: {} | Offline: {} | VMs/Containers: {} | Total: {}",
            online, offline, vms, total);
        let total_width: usize = 130;
        let padding = total_width.saturating_sub(title_str.len() + stats_str.len());

        println!("{}{}{}",
            title_str.green().bold(),
            " ".repeat(padding),
            stats_str
        );

        // Create last scan with version aligned to the right
        let last_scan_str = format!("Last Scan: {}", Utc::now().format("%H:%M:%S"));
        let version_str = format!("v{}", env!("CARGO_PKG_VERSION"));
        let total_width: usize = 130;
        let padding = total_width.saturating_sub(last_scan_str.len() + version_str.len());

        println!("{}{}{}", last_scan_str, " ".repeat(padding), version_str);
        println!("{}", "-".repeat(130));
        println!(
            "{:<15} {:<17} {:<17} {:<20} {:<10} {:<30}",
            "IP".bright_white().bold(),
            "MAC".bright_white().bold(),
            "VIRTUAL MAC".bright_white().bold(),
            "HOSTNAME".bright_white().bold(),
            "STATUS".bright_white().bold(),
            "VENDOR".bright_white().bold()
        );
        println!("{}", "-".repeat(130));
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

        for device in &devices {
            let hostname = device.hostname.as_deref().unwrap_or("").trim_start_matches('_');
            let vendor_display = device.vendor.as_deref().unwrap_or("");

            // Determine if this is a virtual device
            let is_virtual = device.virtual_mac.is_some() ||
                vendor_display.contains("Virtual") ||
                vendor_display.contains("Proxmox") ||
                vendor_display.contains("QEMU") ||
                vendor_display.contains("VMware") ||
                vendor_display.contains("Hyper-V") ||
                vendor_display.contains("VirtualBox");

            // Status coloring: Bright green (online), Orange/Yellow (offline), Red bold (blocked)
            let status_str = device.status.to_string();
            let status_colored = match device.status {
                DeviceStatus::Online => status_str.bright_green(),
                DeviceStatus::Offline => status_str.yellow(),
                DeviceStatus::Block => status_str.red().bold(),
            };

            // MAC display logic with colors (Palette 1 - Professional Soft):
            // - Real MAC (physical): Bright Green (healthy hardware)
            // - Real MAC (VM with mapping): Blue Bold (consistent with VM theme)
            // - Empty (virtual only): Empty string
            // - Virtual MAC: Bright Yellow (soft highlight)

            // Check if the current MAC in device.mac is virtual
            let current_mac_is_virtual = vendor_display.contains("Virtual") ||
                vendor_display.contains("Private");

            let (mac_display, virtual_mac_display) = if let Some(ref vmac) = device.virtual_mac {
                if &device.mac == vmac {
                    // Virtual MAC only, no real MAC known
                    (String::new(), vmac.bright_yellow().to_string())
                } else if current_mac_is_virtual {
                    // Both MACs are virtual (device.mac is the most recent)
                    // Show only the most recent virtual MAC
                    (String::new(), device.mac.bright_yellow().to_string())
                } else {
                    // device.mac is real, vmac is virtual (proper mapping)
                    (device.mac.blue().bold().to_string(), vmac.bright_yellow().to_string())
                }
            } else {
                // No virtual_mac field set
                if current_mac_is_virtual {
                    // MAC is virtual but wasn't moved to virtual_mac field yet
                    (String::new(), device.mac.bright_yellow().to_string())
                } else {
                    // No virtual MAC - physical device
                    (device.mac.bright_green().to_string(), String::new())
                }
            };

            // Hostname coloring: bright white if set, empty if not available
            let hostname_colored = if hostname.is_empty() {
                String::new()
            } else {
                hostname.bright_white().to_string()
            };

            // Vendor coloring (Palette 1 - Professional Soft):
            // - Virtual/VM: Blue (consistent with VM theme)
            // - Known physical: White
            // - Unknown: Empty
            let vendor_truncated = vendor_display.chars().take(30).collect::<String>();
            let vendor_colored = if is_virtual {
                vendor_truncated.blue().to_string()
            } else if vendor_display.is_empty() {
                String::new()
            } else {
                vendor_truncated.white().to_string()
            };

            // Use custom padding for colored strings to fix alignment
            // CRITICAL: Always display device.ip - apply color after ensuring it's not empty
            let final_ip = if device.ip.is_empty() {
                "UNKNOWN".to_string()
            } else {
                device.ip.clone()
            };

            let ip_to_display = if is_virtual {
                final_ip.blue().to_string()
            } else {
                final_ip
            };

            // Use custom padding for colored strings to fix alignment
            println!(
                "{} {} {} {} {} {}",
                pad_colored(ip_to_display, 15),
                pad_colored(mac_display, 17),
                pad_colored(virtual_mac_display, 17),
                pad_colored(hostname_colored.chars().take(20).collect::<String>(), 20),
                pad_colored(status_colored.to_string(), 10),
                vendor_colored.chars().take(30).collect::<String>()
            );
        }

        println!("{}", "-".repeat(130));

        // Color legend (Palette 1 - Professional Soft) - Footer
        println!("{}", "-".repeat(130));
        println!("{}: {} {} | {} {} {} {} | {} {}",
            "Legend".bright_white().bold(),
            "IP:".dimmed(), format!("{} Physical", "□".white()).dimmed(),
            format!("{} VM/Virtual", "□".blue()),
            "|".dimmed(),
            "MAC:".dimmed(), format!("{} Physical", "□".bright_green()).dimmed(),
            format!("{} VM Real", "□".blue().bold()),
            format!("{} VM Virtual", "□".bright_yellow())
        );
    }
}

fn load_devices() -> io::Result<Vec<Device>> {
    if !Path::new(STORAGE_FILE).exists() {
        return Ok(Vec::new());
    }

    let file = File::open(STORAGE_FILE)?;

    // Acquire shared lock for reading
    file.lock_shared().map_err(|e| {
        eprintln!("Failed to acquire read lock on {}: {}", STORAGE_FILE, e);
        e
    })?;

    let reader = BufReader::new(&file);
    let devices: Vec<Device> = serde_json::from_reader(reader)?;

    // Lock is automatically released when file goes out of scope
    file.unlock().ok();

    // Validate all loaded devices and filter out invalid ones
    let total_count = devices.len();
    let valid_devices: Vec<Device> = devices
        .into_iter()
        .filter(|d| {
            let is_valid = d.validate();
            if !is_valid {
                eprintln!(
                    "Skipping corrupted device from {}: IP='{}', MAC='{}'",
                    STORAGE_FILE, d.ip, d.mac
                );
            }
            is_valid
        })
        .collect();

    let filtered_count = total_count - valid_devices.len();
    if filtered_count > 0 {
        eprintln!(
            "WARNING: Filtered out {} corrupted device(s) from {}",
            filtered_count, STORAGE_FILE
        );
    }

    Ok(valid_devices)
}

fn load_blacklist() -> io::Result<Vec<String>> {
    if !Path::new(BLACKLIST_FILE).exists() {
        return Ok(Vec::new());
    }

    let file = File::open(BLACKLIST_FILE)?;

    // Acquire shared lock for reading
    file.lock_shared().map_err(|e| {
        eprintln!("Failed to acquire read lock on {}: {}", BLACKLIST_FILE, e);
        e
    })?;

    let reader = BufReader::new(&file);
    let blacklist: Vec<String> = serde_json::from_reader(reader)?;

    // Lock is automatically released when file goes out of scope
    file.unlock().ok();

    Ok(blacklist)
}

fn save_devices(devices: &[Device]) -> io::Result<()> {
    let file = File::create(STORAGE_FILE)?;

    // Acquire exclusive lock for writing
    file.lock_exclusive().map_err(|e| {
        eprintln!("Failed to acquire write lock on {}: {}", STORAGE_FILE, e);
        eprintln!("This may indicate multiple instances of the monitor are running.");
        eprintln!("Please ensure only one instance is running at a time.");
        e
    })?;

    let writer = BufWriter::new(&file);
    serde_json::to_writer_pretty(writer, devices)?;

    // Explicitly unlock before closing
    file.unlock().ok();

    Ok(())
}

fn notify(summary: &str, body: &str) {
    let _ = Notification::new().summary(summary).body(body).show();
}
