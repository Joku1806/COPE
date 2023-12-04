use super::PacketBuilder;
use super::TGStrategy;

use std::time::SystemTime;

// This Generator generates a new packet after a wait_duration

pub struct TimedGenerator {
    // NOTE: The last timestamp at which a packet was generated
    generation_timestamp: SystemTime,
    wait_duration: std::time::Duration,
    packet_size: usize,
}

impl TimedGenerator {
    pub fn new(wait_duration: std::time::Duration, packet_size: usize) -> Self {
        TimedGenerator {
            generation_timestamp: SystemTime::now(),
            wait_duration,
            packet_size,
        }
    }
}

impl TGStrategy for TimedGenerator {
    fn generate(&mut self) -> Option<PacketBuilder> {
        let elapsed = self.generation_timestamp.elapsed().unwrap();
        if elapsed < self.wait_duration {
            return None;
        }

        self.generation_timestamp = SystemTime::now();

        let packet_builder = PacketBuilder::new().with_data_size(self.packet_size);

        Some(packet_builder)
    }
}
