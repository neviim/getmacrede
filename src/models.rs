use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

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
}

impl Device {
    pub fn new(mac: String, ip: String, hostname: Option<String>, vendor: Option<String>) -> Self {
        Self {
            mac,
            ip,
            hostname,
            vendor,
            last_seen: Utc::now(),
            status: DeviceStatus::Online,
        }
    }
}
