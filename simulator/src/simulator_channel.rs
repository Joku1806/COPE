use cope::packet::Packet;
use cope::stats::Stats;
use cope::{channel::Channel, stats::StatsLogger};
use cope_config::types::node_id::NodeID;
use std::path::Path;
use std::time::Duration;
use std::{
    error::Error,
    io::Write,
    sync::mpsc::{Receiver, Sender},
};

pub struct SimulatorStatsLogger {
    file: std::fs::File,
}

impl StatsLogger for SimulatorStatsLogger {
    fn new(path: &str) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let p = Path::new(path).canonicalize()?;

        if let Some(dirs) = p.parent() {
            std::fs::create_dir_all(dirs)?;
        }

        let file = std::fs::File::create(path)?;

        Ok(Self { file })
    }

    fn log(&mut self, data: &str) {
        if let Err(e) = writeln!(self.file, "{}", data) {
            log::warn!("Could not log data: {}", e);
        }
    }
}

pub struct SimulatorChannel {
    rx: Receiver<Packet>,
    tx: Sender<Packet>,
    stats: Stats,
}

// TODO: Figure out if this is needed
unsafe impl Send for SimulatorChannel {}
unsafe impl Sync for SimulatorChannel {}

impl SimulatorChannel {
    pub fn new(rx: Receiver<Packet>, tx: Sender<Packet>, id: NodeID) -> Self {
        let logger =
            SimulatorStatsLogger::new(format!("./log/simulator/log_{}", id.unwrap()).as_str())
                .unwrap();

        SimulatorChannel {
            rx,
            tx,
            stats: Stats::new(id, Duration::from_secs(30), Box::new(logger)),
        }
    }
}

impl Channel for SimulatorChannel {
    fn transmit(&mut self, packet: &Packet) -> Result<(), Box<dyn Error>> {
        // FIXME: Figure out how to send without cloning
        self.tx.send(packet.clone()).unwrap();

        self.stats.add_send(packet);
        Ok(())
    }

    fn receive(&mut self) -> Option<Packet> {
        // TODO: refactor
        match self.rx.try_recv() {
            Ok(packet) => {
                self.stats.add_rec(&packet);
                return Some(packet);
            }
            Err(_) => None,
        }
    }

    fn log_statistics(&mut self) {
        // FIXME: Figure out in what format the data should be recorded.
        // With this method, we log every 30 seconds and then reset all statistics.
        // Maybe it would be better if we did not reset at all.
        // Instead we could log every time a change happens.
        self.stats.record();
    }
}
