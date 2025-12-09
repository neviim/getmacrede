use std::net::Ipv4Addr;
use std::str::FromStr;
use std::fmt;
use std::process::Command;
use std::fs;
use std::collections::HashMap;
use std::time::Duration;
use dns_lookup::lookup_addr;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;
use tokio::time::timeout;

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat,
    InvalidIp,
    InvalidRange,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid format. Expected format: 192.168.1.1-254"),
            ParseError::InvalidIp => write!(f, "Invalid IP address"),
            ParseError::InvalidRange => write!(f, "Invalid range. Start must be less than or equal to end, and within 0-255"),
        }
    }
}

pub fn parse_ip_range(range_str: &str) -> Result<Vec<Ipv4Addr>, ParseError> {
    // Expected format: "10.10.0.1-254"
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return Err(ParseError::InvalidFormat);
    }

    let start_ip_str = parts[0];
    let end_suffix = parts[1];

    let start_ip = Ipv4Addr::from_str(start_ip_str).map_err(|_| ParseError::InvalidIp)?;
    let octets = start_ip.octets();
    
    let end_octet: u8 = end_suffix.parse().map_err(|_| ParseError::InvalidRange)?;
    let start_octet = octets[3];

    if start_octet > end_octet {
        return Err(ParseError::InvalidRange);
    }

    let mut ips = Vec::new();
    for i in start_octet..=end_octet {
        ips.push(Ipv4Addr::new(octets[0], octets[1], octets[2], i));
    }

    Ok(ips)
}

/// Flush ARP cache to ensure fresh MAC address detection
/// Requires root/sudo privileges
pub fn flush_arp_cache(interface: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let output = if let Some(iface) = interface {
        Command::new("ip")
            .args(&["neigh", "flush", "dev", iface])
            .output()?
    } else {
        Command::new("ip")
            .args(&["neigh", "flush", "all"])
            .output()?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to flush ARP cache: {}", stderr).into());
    }

    Ok(())
}

/// Load DHCP leases from common locations
/// Returns a HashMap of IP -> Hostname mappings
pub fn load_dhcp_leases() -> HashMap<String, String> {
    let mut leases = HashMap::new();

    // Common DHCP lease file locations
    let lease_files = vec![
        "/var/lib/dhcp/dhcpd.leases",           // ISC DHCP Server
        "/var/lib/dhcpd/dhcpd.leases",          // ISC DHCP Server (alternative)
        "/tmp/dhcp.leases",                     // dnsmasq
        "/var/lib/misc/dnsmasq.leases",         // dnsmasq (Debian/Ubuntu)
        "/var/lib/dnsmasq/dnsmasq.leases",      // dnsmasq (alternative)
        "/etc/pihole/dhcp.leases",              // Pi-hole
    ];

    for file_path in lease_files {
        if let Ok(content) = fs::read_to_string(file_path) {
            // Try parsing as dnsmasq format first
            // Format: timestamp mac ip hostname client-id
            // Example: 1234567890 aa:bb:cc:dd:ee:ff 192.168.1.100 mylaptop *
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let ip = parts[2];
                    let hostname = parts[3];
                    if hostname != "*" && !hostname.is_empty() {
                        leases.insert(ip.to_string(), hostname.to_string());
                    }
                }
            }

            // Try parsing as ISC DHCP format
            // Format: lease 192.168.1.100 { ... client-hostname "mylaptop"; ... }
            let mut current_ip: Option<String> = None;
            for line in content.lines() {
                let trimmed = line.trim();

                if trimmed.starts_with("lease ") {
                    // Extract IP from "lease 192.168.1.100 {"
                    if let Some(ip_str) = trimmed.strip_prefix("lease ") {
                        if let Some(ip) = ip_str.split_whitespace().next() {
                            current_ip = Some(ip.to_string());
                        }
                    }
                } else if let Some(ip) = &current_ip {
                    // Look for client-hostname "name";
                    if trimmed.starts_with("client-hostname") {
                        if let Some(name_part) = trimmed.strip_prefix("client-hostname") {
                            let name = name_part
                                .trim()
                                .trim_matches('"')
                                .trim_matches(';')
                                .trim();
                            if !name.is_empty() {
                                leases.insert(ip.clone(), name.to_string());
                            }
                        }
                    }
                } else if trimmed == "}" {
                    current_ip = None;
                }
            }
        }
    }

    leases
}

/// Try NetBIOS name lookup (for Windows devices)
/// Uses nmblookup command if available
fn try_netbios_lookup(ip: &str) -> Option<String> {
    // Try nmblookup -A <ip> to get NetBIOS name
    let output = Command::new("nmblookup")
        .args(&["-A", ip])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse output looking for the computer name
        // Format: "    COMPUTERNAME    <00> -         B <ACTIVE>"
        for line in stdout.lines() {
            if line.contains("<00>") && line.contains("ACTIVE") && !line.contains("GROUP") {
                // Extract the name (first field before <00>)
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if !parts.is_empty() {
                    let name = parts[0].trim();
                    // Filter out special names and non-computer names
                    if !name.starts_with('_') && !name.starts_with("..") && name.len() > 0 {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Resolve hostname using multiple methods: DHCP leases, DNS reverse lookup, mDNS, and NetBIOS
/// This improves hostname detection in home networks where DNS PTR records don't exist
pub async fn resolve_hostname_with_timeout(
    ip: std::net::IpAddr,
    dhcp_leases: Option<&HashMap<String, String>>,
    timeout_seconds: u64,
) -> Option<String> {
    // Wrap the actual resolution in a timeout
    match timeout(
        Duration::from_secs(timeout_seconds),
        resolve_hostname_impl(ip, dhcp_leases),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => None, // Timeout occurred
    }
}

/// Internal implementation of hostname resolution
async fn resolve_hostname_impl(
    ip: std::net::IpAddr,
    dhcp_leases: Option<&HashMap<String, String>>,
) -> Option<String> {
    let ip_str = ip.to_string();

    // 0. Try DHCP leases first (fastest, most reliable for local networks)
    if let Some(leases) = dhcp_leases {
        if let Some(hostname) = leases.get(&ip_str) {
            return Some(hostname.clone());
        }
    }

    // 1. Try standard DNS reverse lookup (fast)
    if let Ok(hostname) = lookup_addr(&ip) {
        // Filter out IP addresses returned as hostnames
        if !hostname.chars().next()?.is_numeric() {
            return Some(hostname);
        }
    }

    // 2. Try mDNS (.local) resolution
    // Create a resolver configured for mDNS
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default()
    );

    // Try querying common .local patterns
    let ip_parts: Vec<String> = ip.to_string().split('.').map(|s| s.to_string()).collect();

    // Try patterns like: ip-192-168-1-100.local
    let patterns = vec![
        format!("ip-{}-{}-{}-{}.local", ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]),
        format!("{}-{}-{}-{}.local", ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]),
    ];

    for pattern in patterns {
        if let Ok(response) = resolver.lookup_ip(pattern.clone()).await {
            if response.iter().any(|resolved_ip| resolved_ip == ip) {
                return Some(pattern.trim_end_matches(".local").to_string());
            }
        }
    }

    // 3. Try NetBIOS lookup (for Windows devices)
    if let Some(netbios_name) = try_netbios_lookup(&ip.to_string()) {
        return Some(netbios_name);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_range() {
        let ips = parse_ip_range("192.168.1.1-5").unwrap();
        assert_eq!(ips.len(), 5);
        assert_eq!(ips[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(ips[4], Ipv4Addr::new(192, 168, 1, 5));
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(parse_ip_range("192.168.1.1").is_err());
    }
}
