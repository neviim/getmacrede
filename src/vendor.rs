use std::collections::HashMap;

/// OUI (Organizationally Unique Identifier) Vendor Lookup
/// The first 3 bytes (24 bits) of a MAC address identify the manufacturer

pub struct VendorDb {
    oui_map: HashMap<String, &'static str>,
}

impl VendorDb {
    pub fn new() -> Self {
        let mut oui_map = HashMap::new();

        // Virtualization / Cloud Vendors
        oui_map.insert("00:15:5D".to_string(), "Microsoft Hyper-V");
        oui_map.insert("00:50:56".to_string(), "VMware");
        oui_map.insert("00:0C:29".to_string(), "VMware");
        oui_map.insert("00:05:69".to_string(), "VMware");
        oui_map.insert("00:1C:14".to_string(), "VMware");
        oui_map.insert("52:54:00".to_string(), "QEMU/KVM Virtual NIC");
        oui_map.insert("BC:24:11".to_string(), "Proxmox Virtual Machine");
        oui_map.insert("00:16:3E".to_string(), "Xen Virtual Machine");
        oui_map.insert("08:00:27".to_string(), "Oracle VirtualBox");
        oui_map.insert("00:21:F6".to_string(), "Oracle VirtualBox");

        // Network Equipment Vendors
        oui_map.insert("00:00:0C".to_string(), "Cisco Systems");
        oui_map.insert("00:01:42".to_string(), "Cisco Systems");
        oui_map.insert("00:01:43".to_string(), "Cisco Systems");
        oui_map.insert("00:01:96".to_string(), "Cisco Systems");
        oui_map.insert("00:01:97".to_string(), "Cisco Systems");
        oui_map.insert("00:01:C7".to_string(), "Cisco Systems");
        oui_map.insert("00:02:3D".to_string(), "Cisco Systems");
        oui_map.insert("00:02:4A".to_string(), "Cisco Systems");
        oui_map.insert("00:02:4B".to_string(), "Cisco Systems");
        oui_map.insert("00:50:F2".to_string(), "Microsoft Corporation");
        oui_map.insert("AC:DE:48".to_string(), "Ubiquiti Networks");
        oui_map.insert("DC:9F:DB".to_string(), "Ubiquiti Networks");
        oui_map.insert("F0:9F:C2".to_string(), "Ubiquiti Networks");
        oui_map.insert("68:D7:9A".to_string(), "Ubiquiti Networks");
        oui_map.insert("24:A4:3C".to_string(), "Ubiquiti Networks");
        oui_map.insert("E4:38:83".to_string(), "TP-Link");
        oui_map.insert("98:DE:D0".to_string(), "TP-Link");
        oui_map.insert("50:C7:BF".to_string(), "TP-Link");
        oui_map.insert("A4:2B:B0".to_string(), "TP-Link");

        // Computer Manufacturers
        oui_map.insert("00:50:B6".to_string(), "Dell");
        oui_map.insert("00:14:22".to_string(), "Dell");
        oui_map.insert("D4:BE:D9".to_string(), "Dell");
        oui_map.insert("D0:67:E5".to_string(), "Dell");
        oui_map.insert("18:03:73".to_string(), "Dell");
        oui_map.insert("00:1B:21".to_string(), "Dell");
        oui_map.insert("00:15:C5".to_string(), "Dell");
        oui_map.insert("B8:CA:3A".to_string(), "Dell");
        oui_map.insert("3C:D9:2B".to_string(), "Hewlett Packard");
        oui_map.insert("00:1F:29".to_string(), "Hewlett Packard");
        oui_map.insert("00:1E:0B".to_string(), "Hewlett Packard");
        oui_map.insert("00:24:81".to_string(), "Hewlett Packard");
        oui_map.insert("00:26:55".to_string(), "Hewlett Packard");
        oui_map.insert("D4:85:64".to_string(), "Hewlett Packard");
        oui_map.insert("EC:B1:D7".to_string(), "Hewlett Packard");
        oui_map.insert("00:03:93".to_string(), "Apple");
        oui_map.insert("00:05:02".to_string(), "Apple");
        oui_map.insert("00:0A:27".to_string(), "Apple");
        oui_map.insert("00:0A:95".to_string(), "Apple");
        oui_map.insert("00:0D:93".to_string(), "Apple");
        oui_map.insert("00:16:CB".to_string(), "Apple");
        oui_map.insert("00:17:F2".to_string(), "Apple");
        oui_map.insert("00:19:E3".to_string(), "Apple");
        oui_map.insert("00:1B:63".to_string(), "Apple");
        oui_map.insert("00:1C:B3".to_string(), "Apple");
        oui_map.insert("00:1D:4F".to_string(), "Apple");
        oui_map.insert("00:1E:52".to_string(), "Apple");
        oui_map.insert("00:1F:5B".to_string(), "Apple");
        oui_map.insert("00:1F:F3".to_string(), "Apple");
        oui_map.insert("00:21:E9".to_string(), "Apple");
        oui_map.insert("00:22:41".to_string(), "Apple");
        oui_map.insert("00:23:12".to_string(), "Apple");
        oui_map.insert("00:23:32".to_string(), "Apple");
        oui_map.insert("00:23:6C".to_string(), "Apple");
        oui_map.insert("00:23:DF".to_string(), "Apple");
        oui_map.insert("00:24:36".to_string(), "Apple");
        oui_map.insert("00:25:00".to_string(), "Apple");
        oui_map.insert("00:25:4B".to_string(), "Apple");
        oui_map.insert("00:25:BC".to_string(), "Apple");
        oui_map.insert("00:26:08".to_string(), "Apple");
        oui_map.insert("00:26:4A".to_string(), "Apple");
        oui_map.insert("00:26:B0".to_string(), "Apple");
        oui_map.insert("00:26:BB".to_string(), "Apple");
        oui_map.insert("04:0C:CE".to_string(), "Apple");
        oui_map.insert("04:15:52".to_string(), "Apple");
        oui_map.insert("0C:3E:9F".to_string(), "Apple");
        oui_map.insert("10:DD:B1".to_string(), "Apple");
        oui_map.insert("18:E7:F4".to_string(), "Apple");
        oui_map.insert("28:CF:E9".to_string(), "Apple");
        oui_map.insert("30:05:5C".to_string(), "Lenovo");
        oui_map.insert("00:21:CC".to_string(), "Lenovo");
        oui_map.insert("00:1F:16".to_string(), "Lenovo");
        oui_map.insert("54:EE:75".to_string(), "Lenovo");
        oui_map.insert("B8:AC:6F".to_string(), "Lenovo");

        // Network Interface Manufacturers
        oui_map.insert("00:E0:4C".to_string(), "Realtek");
        oui_map.insert("00:0C:76".to_string(), "Realtek");
        oui_map.insert("52:54:00".to_string(), "Realtek (or QEMU)");
        oui_map.insert("D8:0D:17".to_string(), "Realtek");
        oui_map.insert("E8:6A:64".to_string(), "Realtek");
        oui_map.insert("00:13:3B".to_string(), "Intel");
        oui_map.insert("00:15:17".to_string(), "Intel");
        oui_map.insert("00:1B:21".to_string(), "Intel");
        oui_map.insert("00:1E:67".to_string(), "Intel");
        oui_map.insert("00:21:5C".to_string(), "Intel");
        oui_map.insert("00:23:15".to_string(), "Intel");
        oui_map.insert("00:25:64".to_string(), "Intel");
        oui_map.insert("68:5B:35".to_string(), "Intel");
        oui_map.insert("D0:94:66".to_string(), "Intel");
        oui_map.insert("00:60:B0".to_string(), "Hewlett Packard");
        oui_map.insert("00:11:0A".to_string(), "Hewlett Packard");
        oui_map.insert("00:15:60".to_string(), "Hewlett Packard");
        oui_map.insert("00:17:A4".to_string(), "Hewlett Packard");
        oui_map.insert("00:1A:4B".to_string(), "Hewlett Packard");
        oui_map.insert("00:1E:0B".to_string(), "Hewlett Packard");
        oui_map.insert("00:21:5A".to_string(), "Hewlett Packard");
        oui_map.insert("00:23:7D".to_string(), "Hewlett Packard");
        oui_map.insert("00:25:B3".to_string(), "Hewlett Packard");
        oui_map.insert("00:26:55".to_string(), "Hewlett Packard");

        // Raspberry Pi
        oui_map.insert("B8:27:EB".to_string(), "Raspberry Pi");
        oui_map.insert("DC:A6:32".to_string(), "Raspberry Pi");
        oui_map.insert("E4:5F:01".to_string(), "Raspberry Pi");

        // Common Routers
        oui_map.insert("24:2F:D0".to_string(), "Intelbras");
        oui_map.insert("64:1C:67".to_string(), "Intelbras");
        oui_map.insert("B0:19:21".to_string(), "Intelbras");
        oui_map.insert("CA:4E:2B".to_string(), "Unknown (Private/Virtual)");

        // Android/Samsung
        oui_map.insert("98:5A:EB".to_string(), "Samsung Electronics");
        oui_map.insert("8C:3B:AD".to_string(), "Samsung Electronics");
        oui_map.insert("44:07:0B".to_string(), "Samsung Electronics");

        Self { oui_map }
    }

    /// Lookup vendor by MAC address
    pub fn lookup(&self, mac: &str) -> Option<String> {
        // Normalize MAC address and extract OUI (first 3 bytes)
        let normalized = mac.to_uppercase().replace("-", ":");
        let parts: Vec<&str> = normalized.split(':').collect();

        if parts.len() < 3 {
            return None;
        }

        // Try exact OUI match (XX:XX:XX)
        let oui = format!("{}:{}:{}", parts[0], parts[1], parts[2]);

        if let Some(vendor) = self.oui_map.get(&oui) {
            return Some(vendor.to_string());
        }

        // Check if it's a locally administered address (virtual/private)
        if let Ok(first_byte) = u8::from_str_radix(parts[0], 16) {
            if first_byte & 0x02 != 0 {
                return Some("Virtual/Private MAC".to_string());
            }
        }

        None
    }

    /// Check if MAC appears to be from a virtual machine
    pub fn is_virtual(&self, mac: &str) -> bool {
        if let Some(vendor) = self.lookup(mac) {
            let vendor_lower = vendor.to_lowercase();
            vendor_lower.contains("virtual")
                || vendor_lower.contains("vmware")
                || vendor_lower.contains("qemu")
                || vendor_lower.contains("kvm")
                || vendor_lower.contains("proxmox")
                || vendor_lower.contains("hyper-v")
                || vendor_lower.contains("xen")
                || vendor_lower.contains("virtualbox")
                || vendor_lower.contains("private")
        } else {
            false
        }
    }
}

impl Default for VendorDb {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_proxmox() {
        let db = VendorDb::new();
        assert_eq!(db.lookup("BC:24:11:36:2D:6E"), Some("Proxmox Virtual Machine".to_string()));
    }

    #[test]
    fn test_lookup_intel() {
        let db = VendorDb::new();
        assert_eq!(db.lookup("68:5B:35:8D:89:41"), Some("Intel".to_string()));
    }

    #[test]
    fn test_is_virtual() {
        let db = VendorDb::new();
        assert!(db.is_virtual("BC:24:11:36:2D:6E")); // Proxmox
        assert!(db.is_virtual("52:54:00:12:34:56")); // QEMU
        assert!(!db.is_virtual("68:5B:35:8D:89:41")); // Intel (physical)
    }
}
