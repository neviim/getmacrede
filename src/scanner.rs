use pnet::datalink::{self, Channel, MacAddr, NetworkInterface};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::models::Device;
use crate::vendor::VendorDb;
use crate::utils;

pub async fn scan_network(
    interface_name: Option<String>,
    target_ips: Vec<Ipv4Addr>,
    resolve_hostnames: bool,
) -> Vec<Device> {
    let vendor_db = VendorDb::new();
    let interface = if let Some(name) = interface_name {
        datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == name)
            .expect("Interface not found")
    } else {
        get_default_interface().expect("No suitable network interface found")
    };

    let _source_mac = interface.mac.expect("Network interface has no MAC address");
    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| ip.ip())
        .expect(&format!("Network interface '{}' has no IPv4 address assigned", interface.name));
    let source_ipv4 = match source_ip {
        std::net::IpAddr::V4(ip) => ip,
        _ => return Vec::new(),
    };

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    let devices = Arc::new(Mutex::new(Vec::<Device>::new()));
    let devices_clone = devices.clone();

    // Receiver Thread
    let rx_thread = thread::spawn(move || {
        let start = Instant::now();
        loop {
            if start.elapsed() > Duration::from_secs(10) {
                break;
            }
            match rx.next() {
                Ok(packet) => {
                    let packet = EthernetPacket::new(packet).unwrap();
                    if packet.get_ethertype() == EtherTypes::Arp {
                        let arp_packet = ArpPacket::new(packet.payload()).unwrap();
                        if arp_packet.get_operation() == ArpOperations::Reply {
                            if arp_packet.get_target_proto_addr() == source_ipv4 {
                                let sender_mac = arp_packet.get_sender_hw_addr();
                                let sender_ip = arp_packet.get_sender_proto_addr();

                                // Vendor lookup using OUI database
                                let mac_str = sender_mac.to_string();
                                let vendor = vendor_db.lookup(&mac_str);

                                // Create device without hostname first (will resolve later)
                                let device = Device::new(
                                    mac_str,
                                    sender_ip.to_string(),
                                    None,
                                    vendor,
                                );

                                let mut devs = devices_clone.lock().unwrap();
                                if !devs.iter().any(|d| d.mac == device.mac) {
                                    devs.push(device);
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }
    });

    // Sender Logic
    for target_ip in target_ips {
        send_arp_request(&mut *tx, &interface, source_ipv4, target_ip);
        // Delay between requests to avoid flooding and packet loss
        thread::sleep(Duration::from_millis(2));
    }

    rx_thread.join().unwrap();

    let mut result = devices.lock().unwrap().clone();

    // Only resolve hostnames if requested (can be slow)
    if resolve_hostnames {
        // Load DHCP leases for faster hostname resolution
        let dhcp_leases = utils::load_dhcp_leases();

        // Resolve hostnames for all discovered devices (async with timeout)
        for device in &mut result {
            if let Ok(ip) = device.ip.parse::<std::net::IpAddr>() {
                // Use 3 second timeout per device to avoid long delays
                if let Some(hostname) = utils::resolve_hostname_with_timeout(
                    ip,
                    Some(&dhcp_leases),
                    3,
                )
                .await
                {
                    device.hostname = Some(hostname);
                }
            }
        }
    }

    result
}

fn get_default_interface() -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| {
            !iface.is_loopback()
            && iface.is_up()
            && iface.mac.is_some()
            && iface.ips.iter().any(|ip| ip.is_ipv4())
        })
}

fn send_arp_request(
    tx: &mut dyn datalink::DataLinkSender,
    interface: &NetworkInterface,
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) {
    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    ethernet_packet.set_destination(MacAddr::broadcast());
    ethernet_packet.set_source(interface.mac.unwrap());
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(interface.mac.unwrap());
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(MacAddr::zero());
    arp_packet.set_target_proto_addr(target_ip);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    tx.send_to(ethernet_packet.packet(), None);
}
