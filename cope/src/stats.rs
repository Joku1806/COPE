use crate::config::CONFIG;
use crate::Packet;
use cope_config::types::node_id::NodeID;
use std::num::Wrapping;

pub trait StatsLogger {
    fn new(path: &str) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn log(&mut self, data: &str);
}

pub struct Stats {
    logger: Box<dyn StatsLogger + Send>,
    creation_time: std::time::Instant,
    packets_sent_to: Vec<(NodeID, Wrapping<u32>)>,
    packets_received_from: Vec<(NodeID, Wrapping<u32>)>,
    packets_sent: Wrapping<u32>,
    packets_received: Wrapping<u32>,
    coded_received: Wrapping<u32>,
    decoded_received: Wrapping<u32>,
    report_received: Wrapping<u32>,
    overheard: Wrapping<u32>,
    total_data_sent: Wrapping<u32>,
    total_data_received: Wrapping<u32>,
}

impl Stats {
    pub fn new(node_id: NodeID, logger: Box<dyn StatsLogger + Send>) -> Self {
        let sent_to: Vec<(NodeID, Wrapping<u32>)> = CONFIG
            .get_rx_whitelist_for(node_id)
            .expect("Config should contain rx whitelist")
            .iter()
            .map(|id| (*id, Wrapping(0u32)))
            .collect();

        let received_from: Vec<(NodeID, Wrapping<u32>)> = CONFIG
            .get_tx_whitelist_for(node_id)
            .expect("Config should contain tx whitelist")
            .iter()
            .map(|id| (*id, Wrapping(0u32)))
            .collect();

        let mut stats = Self {
            logger,
            // TODO: Use Instant or SystemTime?
            creation_time: std::time::Instant::now(),
            packets_sent_to: sent_to,
            packets_received_from: received_from,
            packets_sent: Wrapping(0),
            packets_received: Wrapping(0),
            coded_received: Wrapping(0),
            decoded_received: Wrapping(0),
            report_received: Wrapping(0),
            overheard: Wrapping(0),
            total_data_sent: Wrapping(0),
            total_data_received: Wrapping(0),
        };

        let header = stats.file_header();
        stats.logger.log(&header);

        stats
    }

    fn file_header(&self) -> String {
        let mut header = "".to_owned();

        header.push_str("time_us,");

        for (id, _) in &self.packets_sent_to {
            header.push_str(format!("sent_to_{},", id.unwrap()).as_str());
        }
        for (id, _) in &self.packets_received_from {
            header.push_str(format!("received_from_{},", id.unwrap()).as_str());
        }

        header.push_str("packets_sent,packets_received,total_data_sent,total_data_received");

        header
    }

    pub fn log_data(&mut self) {
        let mut formatted = "".to_owned();

        formatted.push_str(format!("{},", self.creation_time.elapsed().as_micros()).as_str());

        for (_, val) in &self.packets_sent_to {
            formatted.push_str(format!("{},", val).as_str());
        }
        for (_, val) in &self.packets_received_from {
            formatted.push_str(format!("{},", val).as_str());
        }

        formatted.push_str(
            format!(
                "{},{},{},{}",
                self.packets_sent,
                self.packets_received,
                self.total_data_sent,
                self.total_data_received
            )
            .as_str(),
        );

        self.logger.log(&formatted);
    }

    pub fn add_sent(&mut self, packet: &Packet) {
        self.packets_sent += 1;
        self.total_data_sent += packet.data().size() as u32;
    }

    pub fn add_received(&mut self, packet: &Packet) {
        self.packets_received += 1;
        self.total_data_received += packet.data().size() as u32;
        let header_len = packet.coding_header().len();
        match header_len {
            0 => self.report_received += 1,
            1 => (),
            _ => self.coded_received += 1,
        }
    }

    // FIXME: How do we call this function?
    // Channel itself does not have the information to decide
    // whether a packet can be decoded. So we will probably
    // have to provide a method on Channel that can be called
    // from outside code.
    pub fn add_decoded(&mut self) {
        self.decoded_received += 1;
    }

    pub fn add_overheard(&mut self) {
        self.overheard += 1;
    }
}
