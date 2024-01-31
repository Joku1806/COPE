use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use cope::config::CONFIG;
use cope::stats::{Stats, StatsLogger};
use cope::Node;
use simple_logger::SimpleLogger;
use simulator_channel::{SimulatorChannel, SimulatorStatsLogger};

mod simulator_channel;

fn main() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()?;

    let (tx, rx) = channel();
    let mut node_channels = HashMap::new();

    let node_ids = CONFIG.get_node_ids();

    for id in node_ids.iter() {
        let (node_tx, node_rx) = channel();
        node_channels.insert(*id, node_tx);

        let logger =
            SimulatorStatsLogger::new(format!("./log/simulator/log_{}", id.unwrap()).as_str())
                .unwrap();
        let stats = Arc::new(Mutex::new(Stats::new(*id, Box::new(logger))));
        let mut node = Node::new(
            *id,
            Box::new(SimulatorChannel::new(node_rx, tx.clone(), &stats)),
            &stats,
        );

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
