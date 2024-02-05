use crate::config::CONFIG;
use crate::packet::CodingHeader;
use crate::Packet;
use cope_config::types::node_id::NodeID;
use cope_config::types::traffic_generator_type::TrafficGeneratorType;
use std::num::Wrapping;

pub trait StatsLogger {
    fn new(path: &str) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn log(&mut self, data: &str);
}

// TODO: Fields and methods for TX/RX error
pub struct Stats {
    logger: Box<dyn StatsLogger + Send>,
    creation_time: std::time::Instant,
    own_id: NodeID,
    // FIXME: Concept breaks down for coded_sent, since there are multiple receivers
    // and you have to narrow that down to the canonical receiver.
    // But I also don't think there is a good solution to this problem.
    target_id: NodeID,
    traffic_generator: TrafficGeneratorType,
    data_sent: Wrapping<u32>,
    packets_sent: Wrapping<u32>,
    reports_sent: Wrapping<u32>,
    natives_sent: Wrapping<u32>,
    coded_sent: Wrapping<u32>,
    data_received: Wrapping<u32>,
    packets_received: Wrapping<u32>,
    reports_received: Wrapping<u32>,
    natives_received: Wrapping<u32>,
    decoded_received: Wrapping<u32>,
    coded_received: Wrapping<u32>,
    cache_hits: Wrapping<u32>,
    cache_misses: Wrapping<u32>,
}

impl Stats {
    pub fn new(node_id: NodeID, logger: Box<dyn StatsLogger + Send>) -> Self {
        let tg = CONFIG
            .get_generator_type_for(node_id)
            .expect("Config should contain traffic generator type");

        let mut stats = Self {
            logger,
            // TODO: Use Instant or SystemTime?
            creation_time: std::time::Instant::now(),
            own_id: node_id,
            target_id: node_id,
            traffic_generator: tg,
            data_sent: Wrapping(0),
            packets_sent: Wrapping(0),
            reports_sent: Wrapping(0),
            natives_sent: Wrapping(0),
            coded_sent: Wrapping(0),
            data_received: Wrapping(0),
            packets_received: Wrapping(0),
            reports_received: Wrapping(0),
            natives_received: Wrapping(0),
            decoded_received: Wrapping(0),
            coded_received: Wrapping(0),
            cache_hits: Wrapping(0),
            cache_misses: Wrapping(0),
        };

        let header = stats.file_header();
        stats.logger.log(&header);

        stats
    }

    fn file_header(&self) -> String {
        "time_us,node_id,target_id,traffic_generator,data_sent,packets_sent,reports_sent,natives_sent,coded_sent,data_received,packets_received,reports_received,natives_received,decoded_received,coded_received,cache_hits,cache_misses".to_owned()
    }

    pub fn log_data(&mut self) {
        let formatted = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.creation_time.elapsed().as_micros(),
            self.own_id,
            self.target_id,
            self.traffic_generator,
            self.data_sent,
            self.packets_sent,
            self.reports_sent,
            self.natives_sent,
            self.coded_sent,
            self.data_received,
            self.packets_received,
            self.reports_received,
            self.natives_received,
            self.decoded_received,
            self.coded_received,
            self.cache_hits,
            self.cache_misses
        );

        self.logger.log(&formatted);
    }

    pub fn add_sent(&mut self, packet: &Packet) {
        // FIXME: What to do about encoded packets with multiple receivers?
        self.target_id = packet.canonical_receiver().unwrap();
        self.packets_sent += 1;
        self.data_sent += packet.data().len() as u32;

        match packet.coding_header() {
            CodingHeader::Native(_) => self.natives_sent += 1,
            CodingHeader::Encoded(_) => self.coded_sent += 1,
            CodingHeader::Control => self.reports_sent += 1,
        };
    }

    // TODO: Have one single function instead of splitting weirdly between before and after decode.
    // Right now it is not very clear when packets_received should be increased, for example.
    pub fn add_received_before_decode_attempt(&mut self, packet: &Packet) {
        self.target_id = packet.sender();
        self.packets_received += 1;
        // FIXME: Use different data_received fields, depending on the packet being encoded or decoded.
        // self.data_received += packet.data().len() as u32;

        match packet.coding_header() {
            CodingHeader::Native(_) => self.natives_received += 1,
            CodingHeader::Encoded(_) => (),
            CodingHeader::Control => self.reports_received += 1,
        }
    }

    // TODO: Refactor caller to be able to pass Packet in all cases
    pub fn add_received_after_decode_attempt(
        &mut self,
        sender: NodeID,
        data_size: u32,
        decode_successful: bool,
    ) {
        self.target_id = sender;
        self.data_received += data_size;
        self.packets_received += 1;

        match decode_successful {
            true => self.decoded_received += 1,
            false => self.coded_received += 1,
        };
    }

    // TODO: call these functions from inside the cache.
    // I think the 26 branch has an abstraction for the cache, so it should go there.
    pub fn add_cache_hit(&mut self, node: &NodeID) {
        self.target_id = *node;
        self.cache_hits += 1;
    }

    pub fn add_cache_miss(&mut self, node: &NodeID) {
        self.target_id = *node;
        self.cache_misses += 1;
    }
}
