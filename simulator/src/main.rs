use crate::esp::Packet;
use std::sync::mpsc::{channel, Sender};
use std::thread::sleep;

mod esp;
mod firewall;
mod node;

fn main() {
    let (tx, rx) = channel();
    let mut node_channels: Vec<Sender<Packet>> = Vec::new();
    let (node_tx, node_rx) = channel();
    node_channels.push(node_tx);
    let esp = esp::ESP::new(node_rx, tx.clone());
    let node = node::Node::new(
        firewall::Firewall::new(esp),
        node::Address { ip: [127, 0, 0, 1] },
    );

    // create new thread
    std::thread::spawn(move || {
        loop {
            node.send(Packet::from_string("Hello World"));
            while let Some(data) = node.recv() {
                println!("[NODE] Received {:?}", data);
            }
            println!("[NODE] send hi");
            // sleep
            sleep(std::time::Duration::from_secs(1));
        }
    });

    let (node_tx, node_rx) = channel();
    node_channels.push(node_tx);

    let node = node::Node::new(
        firewall::Firewall::new(esp::ESP::new(node_rx, tx)),
        node::Address { ip: [127, 0, 0, 1] },
    );

    // create new thread
    std::thread::spawn(move || {
        loop {
            while let Some(data) = node.recv() {
                println!("[NODE2] Received {:?}", data);
                node.send(Packet::from_string("00000000 RESPONSE"));
                println!("[NODE2] send hi");
            }
            // sleep
            sleep(std::time::Duration::from_secs(1));
        }
    });

    loop {
        if let Ok(data) = rx.try_recv() {
            println!("[SIM] Received {:?} forwarding ...", data);
            // todo do not send packet back to sender
            for node in node_channels.iter() {
                node.send(data.clone()).unwrap();
            }
        }
    }
}
