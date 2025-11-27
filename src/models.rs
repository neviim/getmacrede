use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online,
    Offline,
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "Online"),
            DeviceStatus::Offline => write!(f, "Offline"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub mac: String,
    pub ip: String,
    pub vendor: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub status: DeviceStatus,
    pub is_known: bool,
}

impl Device {
    pub fn new(mac: String, ip: String) -> Self {
        Self {
            mac,
            ip,
            vendor: None,
            last_seen: Utc::now(),
            status: DeviceStatus::Online,
            is_known: false,
        }
    }
}
