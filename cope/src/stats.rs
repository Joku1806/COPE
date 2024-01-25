use crate::config::CONFIG;
use crate::Packet;
use cope_config::types::node_id::NodeID;

pub trait StatsLogger {
    fn new(path: &str) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn log(&mut self, data: &str);
}

pub struct Stats {
    logger: Box<dyn StatsLogger + Send>,
    bench_duration: std::time::Duration,
    time_stamp: std::time::Instant,
    packages_send_to: Vec<(NodeID, u32)>,
    packages_rec_from: Vec<(NodeID, u32)>,
    packets_send: u32,
    packets_rec: u32,
    coded_rec: u32,
    decoded_rec: u32,
    report_rec: u32,
    overhearded: u32,
    total_data_send: u32,
    total_data_rec: u32,
}

impl Stats {
    pub fn new(
        node_id: NodeID,
        duration: std::time::Duration,
        logger: Box<dyn StatsLogger + Send>,
    ) -> Self {
        let send_to: Vec<(NodeID, u32)> = CONFIG
            .get_rx_whitelist_for(node_id)
            .expect("Config should contain rx whitelist")
            .iter()
            .map(|id| (*id, 0u32))
            .collect();

        let rec_from: Vec<(NodeID, u32)> = CONFIG
            .get_tx_whitelist_for(node_id)
            .expect("Config should contain tx whitelist")
            .iter()
            .map(|id| (*id, 0u32))
            .collect();

        let mut stats = Self {
            logger,
            bench_duration: duration,
            time_stamp: std::time::Instant::now(),
            packages_send_to: send_to,
            packages_rec_from: rec_from,
            packets_send: 0,
            packets_rec: 0,
            coded_rec: 0,
            decoded_rec: 0,
            report_rec: 0,
            overhearded: 0,
            total_data_send: 0,
            total_data_rec: 0,
        };

        let header = stats.file_header();
        stats.logger.log(&header);

        stats
    }

    pub fn record(&mut self) {
        let time_elapsed = self.time_stamp.elapsed();
        if time_elapsed > self.bench_duration {
            self.log_data();
            self.reset();
            self.time_stamp = std::time::Instant::now();
        }
    }

    fn file_header(&self) -> String {
        let mut header = "".to_owned();

        header.push_str("time,");

        for (id, _) in &self.packages_send_to {
            header.push_str(format!("send_to_{},", id.unwrap()).as_str());
        }
        for (id, _) in &self.packages_rec_from {
            header.push_str(format!("rec_from_{},", id.unwrap()).as_str());
        }

        header.push_str("packets_send,packets_rec,total_data_send,total_data_rec");

        header
    }

    pub fn log_data(&mut self) {
        let mut formatted = "".to_owned();

        formatted.push_str(format!("{},", self.time_stamp.elapsed().as_secs()).as_str());

        for (_, val) in &self.packages_send_to {
            formatted.push_str(format!("{},", val).as_str());
        }
        for (_, val) in &self.packages_rec_from {
            formatted.push_str(format!("{},", val).as_str());
        }

        formatted.push_str(
            format!(
                "{},{},{},{}",
                self.packets_send, self.packets_rec, self.total_data_send, self.total_data_rec
            )
            .as_str(),
        );

        self.logger.log(&formatted);
    }

    pub fn reset(&mut self) {
        self.time_stamp = std::time::Instant::now();
        for (_, val) in &mut self.packages_send_to {
            *val = 0;
        }
        for (_, val) in &mut self.packages_rec_from {
            *val = 0;
        }
        self.packets_rec = 0;
        self.packets_send = 0;
        self.total_data_send = 0;
        self.total_data_rec = 0;
    }

    pub fn add_send(&mut self, packet: &Packet) {
        self.packets_send += 1;
        self.total_data_send += packet.data().size() as u32;
    }

    pub fn add_rec(&mut self, packet: &Packet) {
        self.packets_rec += 1;
        self.total_data_rec += packet.data().size() as u32;
        let header_len = packet.coding_header().len();
        match header_len {
            0 => self.report_rec += 1,
            1 => (),
            _ => self.coded_rec += 1,
        }
    }

    pub fn add_decoded(&mut self) {
        self.decoded_rec += 1;
    }

    pub fn add_overheard(&mut self) {
        self.overhearded += 1;
    }
}
