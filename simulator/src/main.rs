use std::collections::HashMap;
use std::sync::mpsc::channel;

use cope::config::CONFIG;
use cope::Node;
use simulator_channel::SimulatorChannel;

mod simulator_channel;

fn main() {
    let (tx, rx) = channel();
    let mut node_channels = HashMap::new();

    let node_ids = CONFIG.get_node_ids();

    for id in node_ids.iter() {
        let (node_tx, node_rx) = channel();
        node_channels.insert(*id, node_tx);
        let mut node = Node::new(*id, Box::new(SimulatorChannel::new(node_rx, tx.clone())));

        std::thread::spawn(move || loop {
            node.tick();
        });
    }

    loop {
        if let Ok(packet) = rx.try_recv() {
            let sender = packet.sender();
            for (id, node_tx) in node_channels.iter() {
                if *id == sender {
                    continue;
                }

                // NOTE: Because the simulator channel is implemented using a multi-producer, single-consumer queue,
                // we have to forward the packet to each node individually.
                node_tx.send(packet.clone()).unwrap();
            }
        }
    }
}
