use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

const PROXMOX_CONFIG_FILE: &str = "proxmox.json";
const MAC_MAPPING_FILE: &str = "mac_mapping.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub realm: String,
    pub verify_ssl: bool,
}

impl Default for ProxmoxConfig {
    fn default() -> Self {
        Self {
            host: "192.168.1.1".to_string(),
            port: 8006,
            username: "root".to_string(),
            password: "".to_string(),
            realm: "pam".to_string(),
            verify_ssl: false,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthResponse {
    data: AuthData,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthData {
    ticket: String,
    #[serde(rename = "CSRFPreventionToken")]
    csrf_token: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ResourcesResponse {
    data: Vec<Resource>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Resource {
    #[serde(rename = "type")]
    resource_type: String,
    vmid: Option<u32>,
    node: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct NetworkInterface {
    hwaddr: Option<String>,
    name: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct InterfacesResponse {
    data: Vec<NetworkInterface>,
}

#[allow(dead_code)]
pub struct ProxmoxClient {
    client: Client,
    config: ProxmoxConfig,
    ticket: Option<String>,
    csrf_token: Option<String>,
}

#[allow(dead_code)]
impl ProxmoxClient {
    pub fn new(config: ProxmoxConfig) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(!config.verify_ssl)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            ticket: None,
            csrf_token: None,
        }
    }

    async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://{}:{}/api2/json/access/ticket",
            self.config.host, self.config.port
        );

        let params = [
            ("username", format!("{}@{}", self.config.username, self.config.realm)),
            ("password", self.config.password.clone()),
        ];

        let response = self.client.post(&url).form(&params).send().await?;

        if !response.status().is_success() {
            return Err(format!("Authentication failed: {}", response.status()).into());
        }

        let auth_response: AuthResponse = response.json().await?;
        self.ticket = Some(auth_response.data.ticket);
        self.csrf_token = Some(auth_response.data.csrf_token);

        Ok(())
    }

    async fn get_vms_and_containers(&self) -> Result<Vec<Resource>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://{}:{}/api2/json/cluster/resources?type=vm",
            self.config.host, self.config.port
        );

        let ticket = self.ticket.as_ref().ok_or("Not authenticated")?;

        let response = self
            .client
            .get(&url)
            .header("Cookie", format!("PVEAuthCookie={}", ticket))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get VMs: {}", response.status()).into());
        }

        let resources_response: ResourcesResponse = response.json().await?;
        Ok(resources_response.data)
    }

    async fn get_vm_network_interfaces(
        &self,
        node: &str,
        vmid: u32,
        resource_type: &str,
    ) -> Result<Vec<NetworkInterface>, Box<dyn std::error::Error>> {
        let vm_type = if resource_type == "lxc" { "lxc" } else { "qemu" };

        let url = format!(
            "https://{}:{}/api2/json/nodes/{}/{}/{}/config",
            self.config.host, self.config.port, node, vm_type, vmid
        );

        let ticket = self.ticket.as_ref().ok_or("Not authenticated")?;

        let response = self
            .client
            .get(&url)
            .header("Cookie", format!("PVEAuthCookie={}", ticket))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let config: serde_json::Value = response.json().await?;
        let mut interfaces = Vec::new();

        if let Some(data) = config.get("data") {
            // Para containers LXC e VMs QEMU, procurar por net0, net1, etc
            for i in 0..10 {
                let net_key = format!("net{}", i);
                if let Some(net_value) = data.get(&net_key) {
                    if let Some(net_str) = net_value.as_str() {
                        // Parse hwaddr from string like "name=eth0,bridge=vmbr0,hwaddr=BC:24:11:36:2D:6E,ip=dhcp,type=veth"
                        if let Some(hwaddr) = parse_hwaddr_from_config(net_str) {
                            interfaces.push(NetworkInterface {
                                hwaddr: Some(hwaddr),
                                name: format!("net{}", i),
                            });
                        }
                    }
                }
            }
        }

        Ok(interfaces)
    }

    /// Returns a mapping of detected (virtual) MAC -> real MAC from Proxmox VMs/Containers
    /// This helps correct MAC addresses that are obscured by virtualization layers
    pub async fn get_mac_correction_map(&mut self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        // Authenticate first
        self.authenticate().await?;

        // For now, we'll return an empty map since we need more complex logic
        // to match IPs to VMs. This is a placeholder for future enhancement.
        // Users should use a manual mapping file instead.
        Ok(HashMap::new())
    }

    pub async fn get_all_vm_macs(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Authenticate first
        self.authenticate().await?;

        let mut macs = Vec::new();
        let resources = self.get_vms_and_containers().await?;

        for resource in resources {
            if let (Some(vmid), Some(node)) = (resource.vmid, resource.node) {
                let interfaces = self
                    .get_vm_network_interfaces(&node, vmid, &resource.resource_type)
                    .await?;

                for interface in interfaces {
                    if let Some(mac) = interface.hwaddr {
                        let normalized_mac = normalize_mac(&mac);
                        macs.push(normalized_mac);
                    }
                }
            }
        }

        Ok(macs)
    }
}

#[allow(dead_code)]
fn parse_hwaddr_from_config(config_str: &str) -> Option<String> {
    for part in config_str.split(',') {
        let kv: Vec<&str> = part.split('=').collect();
        if kv.len() == 2 && kv[0].trim() == "hwaddr" {
            return Some(kv[1].trim().to_string());
        }
    }
    None
}

fn normalize_mac(mac: &str) -> String {
    mac.to_uppercase()
        .replace("-", ":")
        .chars()
        .collect::<String>()
}

#[allow(dead_code)]
pub fn load_proxmox_config() -> Result<ProxmoxConfig, Box<dyn std::error::Error>> {
    if !Path::new(PROXMOX_CONFIG_FILE).exists() {
        return Err("Proxmox config file not found".into());
    }
    let file = File::open(PROXMOX_CONFIG_FILE)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

#[allow(dead_code)]
pub fn save_proxmox_config_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProxmoxConfig::default();
    let file = File::create("proxmox.json.example")?;
    serde_json::to_writer_pretty(file, &config)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacMapping {
    pub ip: String,
    pub real_mac: String,
    pub description: Option<String>,
}

/// Load manual MAC address mappings from file
/// Format: [{"ip": "192.168.15.31", "real_mac": "BC:24:11:36:2D:6E", "description": "Proxmox Container"}]
pub fn load_mac_mappings() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    if !Path::new(MAC_MAPPING_FILE).exists() {
        return Ok(HashMap::new());
    }
    let file = File::open(MAC_MAPPING_FILE)?;
    let reader = BufReader::new(file);
    let mappings: Vec<MacMapping> = serde_json::from_reader(reader)?;

    let mut map = HashMap::new();
    for mapping in mappings {
        map.insert(mapping.ip, normalize_mac(&mapping.real_mac));
    }

    Ok(map)
}

#[allow(dead_code)]
pub fn save_mac_mapping_example() -> Result<(), Box<dyn std::error::Error>> {
    let example = vec![
        MacMapping {
            ip: "192.168.15.31".to_string(),
            real_mac: "BC:24:11:36:2D:6E".to_string(),
            description: Some("Proxmox LXC Container".to_string()),
        },
    ];
    let file = File::create("mac_mapping.json.example")?;
    serde_json::to_writer_pretty(file, &example)?;
    Ok(())
}
