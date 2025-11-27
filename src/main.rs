mod models;
mod scanner;
mod storage;

use clap::Parser;
use std::thread;
use std::time::Duration;
use chrono::Utc;
use notify_rust::Notification;
use models::{Device, DeviceStatus};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Network interface to use (optional, auto-detected if not provided)
    #[arg(short, long)]
    interface: Option<String>,

    /// Scan interval in seconds
    #[arg(short, long, default_value_t = 30)]
    interval: u64,
}

fn notify_new_device(device: &Device) {
    let summary = format!("New Device Detected: {}", device.ip);
    let body = format!("MAC: {}\nVendor: Unknown", device.mac); // Vendor lookup to be added
    let _ = Notification::new()
        .summary(&summary)
        .body(&body)
        .show();
    println!("NOTIFICATION: {} - {}", summary, body);
}

fn main() {
    let args = Args::parse();

    // 1. Setup Interface
    let interface = if let Some(iface_name) = args.interface {
        pnet::datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == iface_name)
            .expect("Interface not found")
    } else {
        scanner::get_default_interface().expect("No suitable network interface found")
    };

    println!("Using interface: {} ({:?})", interface.name, interface.ips);

    // 2. Load Known Devices
    let mut known_devices = match storage::load_devices() {
        Ok(devices) => devices,
        Err(e) => {
            eprintln!("Failed to load devices: {}", e);
            Vec::new()
        }
    };

    println!("Loaded {} known devices.", known_devices.len());

    loop {
        println!("Scanning network...");
        let scan_results = scanner::scan_network(&interface);
        println!("Scan complete. Found {} devices.", scan_results.len());

        let mut changes = false;

        // Process Scan Results
        for scanned_device in scan_results {
            if let Some(existing_device) = known_devices.iter_mut().find(|d| d.mac == scanned_device.mac) {
                // Update existing device
                if existing_device.status == DeviceStatus::Offline {
                    println!("Device came online: {} ({})", existing_device.ip, existing_device.mac);
                    existing_device.status = DeviceStatus::Online;
                    changes = true;
                }
                existing_device.last_seen = Utc::now();
                existing_device.ip = scanned_device.ip.clone(); // IP might change
            } else {
                // New Device
                println!("New device detected: {} ({})", scanned_device.ip, scanned_device.mac);
                notify_new_device(&scanned_device);
                let mut new_device = scanned_device.clone();
                new_device.is_known = true; // Mark as known after detection
                known_devices.push(new_device);
                changes = true;
            }
        }

        // Check for Offline Devices
        let now = Utc::now();
        for device in known_devices.iter_mut() {
            if device.status == DeviceStatus::Online {
                // If not seen in the last 2 intervals (plus a buffer), mark as offline
                // Using 2.5 * interval as threshold
                let threshold = chrono::Duration::seconds((args.interval as i64 * 2) + 10);
                if now.signed_duration_since(device.last_seen) > threshold {
                    println!("Device went offline: {} ({})", device.ip, device.mac);
                    device.status = DeviceStatus::Offline;
                    changes = true;
                }
            }
        }

        if changes {
            if let Err(e) = storage::save_devices(&known_devices) {
                eprintln!("Failed to save devices: {}", e);
            }
        }

        thread::sleep(Duration::from_secs(args.interval));
    }
}
