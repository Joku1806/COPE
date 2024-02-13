use cope::{config::CONFIG, stats::StatsLogger};
use cope_config::types::mac_address::MacAddress;
use std::num::Wrapping;

pub struct EspNowStats {
    logger: Box<dyn StatsLogger + Send>,
    creation_time: std::time::Instant,
    last_log: std::time::Instant,
    log_frequency: std::time::Duration,
    own_mac: MacAddress,
    data_sent: Wrapping<usize>,
    data_received: Wrapping<usize>,
    data_sniffed: Wrapping<usize>,
    tx_failures: Wrapping<usize>,
}

impl EspNowStats {
    pub fn new(
        mac: MacAddress,
        logger: Box<dyn StatsLogger + Send>,
        log_frequency: std::time::Duration,
    ) -> Self {
        let mut stats = Self {
            logger,
            creation_time: std::time::Instant::now(),
            last_log: std::time::Instant::now(),
            log_frequency,
            own_mac: mac,
            data_sent: Wrapping(0),
            data_received: Wrapping(0),
            data_sniffed: Wrapping(0),
            tx_failures: Wrapping(0),
        };

        if CONFIG.log_espnow_stats {
            let header = stats.file_header();
            stats.logger.log(&header);
        }

        stats
    }

    fn file_header(&self) -> String {
        "time_us,own_mac,data_sent,data_received,data_sniffed,rx_failures,tx_failures".to_owned()
    }

    pub fn log_data(&mut self) {
        if self.last_log.elapsed() < self.log_frequency || !CONFIG.log_espnow_stats {
            return;
        }

        self.last_log = std::time::Instant::now();

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
