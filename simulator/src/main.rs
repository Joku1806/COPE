use std::collections::HashMap;
use std::sync::mpsc::channel;

use cope::config::CONFIG;
use cope::stats::{Stats, StatsLogger};
use cope::Node;
use simple_logger::SimpleLogger;
use simulator_channel::SimulatorChannel;
use simulator_stats_logger::SimulatorStatsLogger;

mod simulator_channel;
mod simulator_stats_logger;

fn main() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
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
        let stats = Stats::new(*id, Box::new(logger), CONFIG.stats_log_duration);

        let mut node = Node::new(
            *id,
            Box::new(SimulatorChannel::new(node_rx, tx.clone())),
            stats,
        );

        std::thread::spawn(move || loop {
            node.tick();
        });
    }

    loop {
        let packet = rx.recv().unwrap();
        let sender = packet.sender();
        for (id, node_tx) in node_channels.iter() {
            if *id == sender {
                continue;
            }

            if CONFIG.simulator_packet_loss > 0.0 {
                let r = rand::random::<f64>();
                if r < CONFIG.simulator_packet_loss {
                    log::info!("Dropping packet from {} to {}", sender, id);
                    continue;
                }
            }

            // NOTE: Because the simulator channel is implemented using a multi-producer, single-consumer queue,
            // we have to forward the packet to each node individually.
            if let Err(e) = node_tx.send(packet.clone()) {
                panic!("{}", e);
            }
        }
    }
}
