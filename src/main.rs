mod models;
mod monitor;
mod proxmox;
mod scanner;
mod utils;
mod vendor;

use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform a single scan of the network
    Scan {
        /// IP range to scan (e.g., 10.10.0.1-254)
        #[arg(short, long)]
        range: String,

        /// Network interface to use
        #[arg(short, long)]
        interface: Option<String>,

        /// Resolve hostnames (slower but shows device names)
        #[arg(long)]
        hostname: bool,
    },
    /// Monitor the network for changes
    Monitor {
        /// IP range to monitor (e.g., 10.10.0.1-254)
        #[arg(short, long)]
        range: String,

        /// Network interface to use
        #[arg(short, long)]
        interface: Option<String>,

        /// Scan interval in seconds
        #[arg(short = 'n', long, default_value_t = 30)]
        interval: u64,

        /// Resolve hostnames (slower but shows device names)
        #[arg(long)]
        hostname: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { range, interface, hostname } => {
            let target_ips = match utils::parse_ip_range(&range) {
                Ok(ips) => ips,
                Err(e) => {
                    eprintln!("Error parsing IP range: {}", e);
                    return;
                }
            };
            println!("GetMacRede v{}", env!("CARGO_PKG_VERSION"));
            println!("Scanning {} IPs...", target_ips.len());
            let devices = scanner::scan_network(interface, target_ips, hostname).await;

            println!(
                "{:<15} {:<17} {:<20} {:<10}",
                "IP", "MAC", "HOSTNAME", "STATUS"
            );
            println!("{}", "-".repeat(65));
            for device in devices {
                let hostname = device.hostname.unwrap_or_else(|| "-".to_string());
                println!(
                    "{:<15} {:<17} {:<20} {:<10}",
                    device.ip,
                    device.mac,
                    hostname.chars().take(20).collect::<String>(),
                    device.status.to_string().green()
                );
            }
        }
        Commands::Monitor {
            range,
            interface,
            interval,
            hostname,
        } => {
            monitor::run_monitor(interface, range, interval, hostname).await;
        }
    }
}
