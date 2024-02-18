use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use cope::config::CONFIG;
use cope::stats::{Stats, StatsLogger};
use cope::Node;
use rand::Rng;
use simple_logger::SimpleLogger;
use simulator_channel::SimulatorChannel;
use simulator_stats_logger::SimulatorStatsLogger;

mod simulator_channel;
mod simulator_stats_logger;

fn main() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;

    let (tx, rx) = channel();
    let mut node_channels = HashMap::new();

    let node_ids = CONFIG.get_node_ids();
    let mut handles = vec![];

    let finished = Arc::new(AtomicBool::new(false));

    for id in node_ids.iter() {
        let (node_tx, node_rx) = channel();
        node_channels.insert(*id, node_tx);

        let logger = SimulatorStatsLogger::new(
            format!(
                "./log/node_{}_{:X}",
                id.unwrap(),
                rand::thread_rng().gen::<u64>()
            )
            .as_str(),
        )
        .unwrap();
        let stats = Stats::new(*id, Box::new(logger), CONFIG.stats_log_duration);

        let mut node = Node::new(
            *id,
            Box::new(SimulatorChannel::new(node_rx, tx.clone())),
            stats,
        );
        let bench_path = format!("./log/bench/log_{}", id);
        node.set_bench_log_path(&bench_path);

        let handle = std::thread::spawn({
            let finished_clone = finished.clone();
            move || loop {
                if finished_clone.load(Ordering::SeqCst) {
                    break;
                }

                node.tick();
            }
        });
        handles.push(handle);
    }

    let start = SystemTime::now();
    let runtime = Duration::from_secs(10);

    loop {
        let elapsed = match start.elapsed() {
            Ok(e) => e,
            Err(_) => Duration::ZERO,
        };

        if elapsed > runtime {
            finished.store(true, Ordering::SeqCst);
            break;
        }

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

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
