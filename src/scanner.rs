use pnet::datalink::{self, Channel, NetworkInterface, MacAddr};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};
use pnet::ipnetwork::IpNetwork;
use std::net::Ipv4Addr;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, Sender, Receiver};

use crate::models::{Device, DeviceStatus};

pub fn get_default_interface() -> Option<NetworkInterface> {
    let interfaces = datalink::interfaces();
    // Simple heuristic: find the first non-loopback interface that is up and has an IP
    interfaces
        .into_iter()
        .find(|iface| !iface.is_loopback() && iface.is_up() && !iface.ips.is_empty())
}

pub fn scan_network(interface: &NetworkInterface) -> Vec<Device> {
    let mut devices = Vec::new();
    let source_mac = interface.mac.unwrap();
    let source_ip = interface.ips.iter().find(|ip| ip.is_ipv4()).map(|ip| ip.ip()).unwrap();
    let source_ipv4 = match source_ip {
        std::net::IpAddr::V4(ip) => ip,
        _ => return devices,
    };

    let network = interface.ips.iter().find(|ip| ip.is_ipv4()).unwrap();
    let network_ipv4 = match network {
        IpNetwork::V4(net) => net,
        _ => return devices,
    };

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e),
    };

    // Spawn a thread to send ARP requests
    let tx_interface = interface.clone();
    let tx_network = network_ipv4.clone();
    thread::spawn(move || {
        for target_ip in tx_network.iter() {
            if target_ip == source_ipv4 {
                continue;
            }
            send_arp_request(&mut *tx, &tx_interface, source_ipv4, target_ip);
            thread::sleep(Duration::from_millis(10)); // Rate limit
        }
    });

    // Listen for ARP replies for a short duration
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(5);

    loop {
        if start_time.elapsed() > timeout {
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
                             
                             devices.push(Device::new(sender_mac.to_string(), sender_ip.to_string()));
                         }
                    }
                }
            }
            Err(_) => {
                // Ignore errors
            }
        }
    }

    devices
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
