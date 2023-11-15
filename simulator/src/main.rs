use std::collections::HashMap;
use crate::esp::Packet;
use std::sync::mpsc::channel;
use std::thread::sleep;

mod esp;
mod firewall;
mod node;

fn main() {
    // init simulator
    let (tx, rx) = channel();
    let mut node_channels = HashMap::new();


    // init new node 111111
    let (node_tx, node_rx) = channel();
    let node_mac = [1,1,1,1,1,1];
    node_channels.insert(node_mac, node_tx);
    let esp = esp::ESP::new(node_rx, tx.clone(), node_mac);
    let node = node::Node::new(
        firewall::Firewall::new(esp), [127, 0, 0, 1] ,
    );

    // create new thread
    std::thread::spawn(move || {
        loop {
            node.send(Packet::from_string("Hello World"));
            println!("[NODE]  send Hello World");
            while let Some(data) = node.recv() {
                println!("[NODE]  Received {:?}", String::from_utf8_lossy(&data));
            }
            // sleep
            sleep(std::time::Duration::from_secs(1));
        }
    });
    // init other node 2:2:2:2:2:2
    let (node_tx, node_rx) = channel();
    let node_mac = [2,2,2,2,2,2];
    node_channels.insert(node_mac, node_tx);

    let node = node::Node::new(
        firewall::Firewall::new(esp::ESP::new(node_rx, tx, node_mac)),
        [127, 0, 0, 1],
    );

    // create new thread
    std::thread::spawn(move || {
        loop {
            while let Some(data) = node.recv() {
                println!("[NODE2] Received {:?}", String::from_utf8_lossy(&data));
                node.send(Packet::from_string("00000000 RESPONSE"));
                println!("[NODE2] 00000000 RESPONSE");
            }
            // sleep
            sleep(std::time::Duration::from_millis(10));
        }
    });

    // networking loop
    loop {
        if let Ok(data) = rx.try_recv() {
            //println!("[SIM] Received {:?} forwarding ...", data);
            // todo do not send packet back to sender
            for (mac, node) in node_channels.iter() {
                if *mac == data.origin {
                    continue;
                }
                node.send(data.clone()).unwrap();
            }
        }
    }
}
