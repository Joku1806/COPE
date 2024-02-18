use cope::{config::CONFIG, stats::StatsLogger};
use cope_config::types::mac_address::MacAddress;
use std::num::Wrapping;

pub struct EspNowStats {
    logger: Box<dyn StatsLogger + Send>,
    creation_time: std::time::Instant,
    last_log: std::time::Instant,
    log_frequency: std::time::Duration,
    own_mac: MacAddress,
    packets_sent: Wrapping<usize>,
    packet_data_sent: Wrapping<usize>,
    raw_frames_sent: Wrapping<usize>,
    raw_data_sent: Wrapping<usize>,
    tx_failures: Wrapping<usize>,
    raw_frames_dropped: Wrapping<usize>,
    raw_data_dropped: Wrapping<usize>,
    raw_frames_received: Wrapping<usize>,
    raw_data_received: Wrapping<usize>,
    packets_dropped: Wrapping<usize>,
    packet_data_dropped: Wrapping<usize>,
    packets_received: Wrapping<usize>,
    packet_data_received: Wrapping<usize>,
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
            packets_sent: Wrapping(0),
            packet_data_sent: Wrapping(0),
            raw_frames_sent: Wrapping(0),
            raw_data_sent: Wrapping(0),
            tx_failures: Wrapping(0),
            raw_frames_dropped: Wrapping(0),
            raw_data_dropped: Wrapping(0),
            raw_frames_received: Wrapping(0),
            raw_data_received: Wrapping(0),
            packets_dropped: Wrapping(0),
            packet_data_dropped: Wrapping(0),
            packets_received: Wrapping(0),
            packet_data_received: Wrapping(0),
        };

        if CONFIG.log_espnow_stats {
            let header = stats.file_header();
            stats.logger.log(&header);
        }

        stats
    }

    fn file_header(&self) -> String {
        "time_us,own_mac,packets_sent,packet_data_sent,raw_frames_sent,raw_data_sent,tx_failures,raw_frames_dropped,raw_data_dropped,raw_frames_received,raw_data_received,packets_dropped,packet_data_dropped,packets_received,packet_data_received".to_owned()
    }

    pub fn log_data(&mut self) {
        if self.last_log.elapsed() < self.log_frequency || !CONFIG.log_espnow_stats {
            return;
        }

        self.last_log = std::time::Instant::now();

        let formatted = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.creation_time.elapsed().as_micros(),
            self.own_mac,
            self.packets_sent,
            self.packet_data_sent,
            self.raw_frames_sent,
            self.raw_data_sent,
            self.tx_failures,
            self.raw_frames_dropped,
            self.raw_data_dropped,
            self.raw_frames_received,
            self.raw_data_received,
            self.packets_dropped,
            self.packet_data_dropped,
            self.packets_received,
            self.packet_data_received,
        );

        self.logger.log(&formatted);
    }

    pub fn add_packet_sent(&mut self) {
        self.packets_sent += 1;
    }

    pub fn add_packet_data_sent(&mut self, bytes: usize) {
        self.packet_data_sent += bytes;
    }

    pub fn add_raw_frame_sent(&mut self) {
        self.raw_frames_sent += 1;
    }

    pub fn add_raw_data_sent(&mut self, bytes: usize) {
        self.raw_data_sent += bytes;
    }

    pub fn add_tx_failure(&mut self) {
        self.tx_failures += 1;
    }

    pub fn add_raw_frame_dropped(&mut self) {
        self.raw_frames_dropped += 1;
    }

    pub fn add_raw_data_dropped(&mut self, bytes: usize) {
        self.raw_data_dropped += bytes;
    }

    pub fn add_raw_frame_received(&mut self) {
        self.raw_frames_received += 1;
    }

    pub fn add_raw_data_received(&mut self, bytes: usize) {
        self.raw_data_received += bytes;
    }

    pub fn add_packet_dropped(&mut self) {
        self.packets_dropped += 1;
    }

    pub fn add_packet_data_dropped(&mut self, bytes: usize) {
        self.packet_data_dropped += bytes;
    }

    pub fn add_packet_received(&mut self) {
        self.packets_received += 1;
    }

    pub fn add_packet_data_received(&mut self, bytes: usize) {
        self.packet_data_received += bytes;
    }
}
