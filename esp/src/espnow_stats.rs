use cope::stats::StatsLogger;
use cope_config::types::mac_address::MacAddress;
use std::num::Wrapping;

pub struct EspNowStats {
    logger: Box<dyn StatsLogger + Send>,
    creation_time: std::time::Instant,
    own_mac: MacAddress,
    data_sent: Wrapping<usize>,
    data_received: Wrapping<usize>,
    data_sniffed: Wrapping<usize>,
    tx_failures: Wrapping<usize>,
}

impl EspNowStats {
    pub fn new(mac: MacAddress, logger: Box<dyn StatsLogger + Send>) -> Self {
        let mut stats = Self {
            logger,
            creation_time: std::time::Instant::now(),
            own_mac: mac,
            data_sent: Wrapping(0),
            data_received: Wrapping(0),
            data_sniffed: Wrapping(0),
            tx_failures: Wrapping(0),
        };

        let header = stats.file_header();
        stats.logger.log(&header);

        stats
    }

    fn file_header(&self) -> String {
        "time_us,own_mac,data_sent,data_received,data_sniffed,rx_failures,tx_failures".to_owned()
    }

    pub fn log_data(&mut self) {
        let formatted = format!(
            "{},{},{},{},{},{}",
            self.creation_time.elapsed().as_micros(),
            self.own_mac,
            self.data_sent,
            self.data_received,
            self.data_sniffed,
            self.tx_failures,
        );

        self.logger.log(&formatted);
    }

    pub fn add_sent(&mut self, bytes: usize) {
        self.data_sent += bytes;
    }

    pub fn add_received(&mut self, bytes: usize) {
        self.data_received += bytes;
    }

    pub fn add_sniffed(&mut self, bytes: usize) {
        self.data_sniffed += bytes;
    }

    pub fn add_tx_failure(&mut self) {
        self.tx_failures += 1;
    }
}
