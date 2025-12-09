use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online,
    Offline,
    Block,
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "Online"),
            DeviceStatus::Offline => write!(f, "Offline"),
            DeviceStatus::Block => write!(f, "Block"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub mac: String,
    pub ip: String,
    pub hostname: Option<String>,
    pub vendor: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub status: DeviceStatus,
    /// Virtual MAC address (e.g., from Proxmox bridge/veth) if different from real MAC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_mac: Option<String>,
}

impl Device {
    pub fn new(mac: String, ip: String, hostname: Option<String>, vendor: Option<String>) -> Self {
        // Validate that IP is actually an IP address, not a MAC
        if !Self::is_valid_ip(&ip) {
            eprintln!("WARNING: Invalid IP detected: '{}' (MAC was: '{}')", ip, mac);
            eprintln!("This device will be skipped to prevent data corruption.");
        }

        // Validate that MAC is actually a MAC address, not an IP
        if !Self::is_valid_mac(&mac) {
            eprintln!("WARNING: Invalid MAC detected: '{}' (IP was: '{}')", mac, ip);
        }

        Self {
            mac,
            ip,
            hostname,
            vendor,
            last_seen: Utc::now(),
            status: DeviceStatus::Online,
            virtual_mac: None,
        }
    }

    /// Validates if a string is a valid IPv4 address
    pub fn is_valid_ip(ip: &str) -> bool {
        ip.parse::<Ipv4Addr>().is_ok()
    }

    /// Validates if a string is a valid MAC address format
    pub fn is_valid_mac(mac: &str) -> bool {
        if mac.is_empty() {
            return false;
        }

        // MAC should be in format XX:XX:XX:XX:XX:XX
        let parts: Vec<&str> = mac.split(':').collect();
        if parts.len() != 6 {
            return false;
        }

        // Each part should be exactly 2 hex characters
        parts.iter().all(|part| {
            part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())
        })
    }

    /// Validates the device data integrity
    pub fn validate(&self) -> bool {
        let ip_valid = Self::is_valid_ip(&self.ip);
        let mac_valid = Self::is_valid_mac(&self.mac);

        if !ip_valid {
            eprintln!("VALIDATION ERROR: Device has invalid IP: '{}' (MAC: '{}')", self.ip, self.mac);
        }
        if !mac_valid {
            eprintln!("VALIDATION ERROR: Device has invalid MAC: '{}' (IP: '{}')", self.mac, self.ip);
        }

        ip_valid && mac_valid
    }
}
