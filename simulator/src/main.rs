use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;

use cope::Node;
use simulator_channel::SimulatorChannel;
mod simulator_channel;

fn main() {
    let (tx, rx) = channel();
    let mut node_channels = HashMap::new();

    let (node_tx, node_rx) = channel();
    node_channels.insert('A', node_tx);
    let node: Node = Node::new(
        'A',
        'C',
        Vec::from(['B']),
        // NOTE: Grrrr heap allocations.
        // I could not get this to work using lifetimes,
        // Apparently you should not store references in structs.
        // But this should not be a problem on the ESP,
        // we have enough heap space for this.
        Box::new(SimulatorChannel::new(node_rx, tx.clone())),
    );

    std::thread::spawn(move || loop {
        node.tick();
        sleep(Duration::from_secs(1));
    });

    let (node_tx, node_rx) = channel();
    node_channels.insert('B', node_tx);
    let node: Node = Node::new(
        'B',
        'C',
        Vec::from(['A']),
        Box::new(SimulatorChannel::new(node_rx, tx.clone())),
    );

    std::thread::spawn(move || loop {
        node.tick();
        sleep(Duration::from_millis(500));
    });

    loop {
        if let Ok(packet) = rx.try_recv() {
            let origin = packet.origin;

            for (id, node_tx) in node_channels.iter() {
                if *id == origin {
                    continue;
                }

                // NOTE: Because the simulator channel is implemented using a multi-producer, single-consumer queue,
                // we have to forward the packet to each node individually.
                node_tx.send(packet.clone()).unwrap();
            }
        }
    }
}
