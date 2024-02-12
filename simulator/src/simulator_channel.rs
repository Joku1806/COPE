use cope::packet::Packet;
use cope::{channel::Channel, stats::StatsLogger};
use std::fs::OpenOptions;
use std::io::LineWriter;
use std::path::Path;
use std::{
    error::Error,
    io::Write,
    sync::mpsc::{Receiver, Sender},
};

pub struct SimulatorStatsLogger {
    line_buffer: LineWriter<std::fs::File>,
}

impl StatsLogger for SimulatorStatsLogger {
    fn new(path: &str) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let p = Path::new(path);

        if let Some(dirs) = p.parent() {
            std::fs::create_dir_all(dirs)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;
        let line_buffer = LineWriter::new(file);

        Ok(Self { line_buffer })
    }

    fn log(&mut self, data: &str) {
        log::debug!("Logging {} to {:?}", data, self.line_buffer);

        if let Err(e) = writeln!(self.line_buffer, "{}", data) {
            log::warn!("Could not log or flush data: {}", e);
        }
    }
}

pub struct SimulatorChannel {
    rx: Receiver<Packet>,
    tx: Sender<Packet>,
}

// TODO: Figure out if this is needed
unsafe impl Send for SimulatorChannel {}
unsafe impl Sync for SimulatorChannel {}

impl SimulatorChannel {
    pub fn new(rx: Receiver<Packet>, tx: Sender<Packet>) -> Self {
        SimulatorChannel { rx, tx }
    }
}

impl Channel for SimulatorChannel {
    fn transmit(&mut self, packet: &Packet) -> Result<(), Box<dyn Error>> {
        // FIXME: Figure out how to send without cloning
        if let Err(e) = self.tx.send(packet.clone()) {
            println!("{}", e);
        }

        Ok(())
    }

    fn receive(&mut self) -> Option<Packet> {
        self.rx.try_recv().ok()
    }
}
