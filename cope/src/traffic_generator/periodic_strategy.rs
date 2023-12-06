use super::PacketBuilder;
use super::TGStrategy;

use std::time::SystemTime;

// This Generator generates a new packet after a wait_duration

pub struct PeriodicStrategy {
    // NOTE: The last timestamp at which a packet was generated
    generation_timestamp: SystemTime,
    wait_duration: std::time::Duration,
}

impl PeriodicStrategy {
    pub fn new(wait_duration: std::time::Duration) -> Self {
        PeriodicStrategy {
            generation_timestamp: SystemTime::now(),
            wait_duration,
        }
    }
}

impl TGStrategy for PeriodicStrategy {
    fn generate(&mut self) -> Option<PacketBuilder> {
        let elapsed = self.generation_timestamp.elapsed().unwrap();
        if elapsed < self.wait_duration {
            return None;
        }

        self.generation_timestamp = SystemTime::now();
        const PACKET_SIZE: usize = 128;
        Some(PacketBuilder::new().with_data_size(PACKET_SIZE))
    }
}
