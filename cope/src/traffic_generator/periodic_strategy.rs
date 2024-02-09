use super::data_generator::DataGenerator;
use super::size_distribution::SizeDistribution;
use super::PacketBuilder;
use super::TGStrategy;

use std::time::SystemTime;

// This Generator generates a new packet after a wait_duration

pub struct PeriodicStrategy {
    // NOTE: The last timestamp at which a packet was generated
    generation_timestamp: SystemTime,
    wait_duration: std::time::Duration,
    // NOTE: Distribution of packet sizes
    size_distribution: SizeDistribution,
    data_generator: DataGenerator,
}

impl PeriodicStrategy {
    pub fn new(wait_duration: std::time::Duration) -> Self {
        PeriodicStrategy {
            generation_timestamp: SystemTime::now(),
            wait_duration,
            size_distribution: SizeDistribution::new(),
            data_generator: DataGenerator::new(),
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
        let target_size = self.size_distribution.sample(&mut rand::thread_rng());
        Some(PacketBuilder::new().data_raw(self.data_generator.generate(target_size)))
    }
}
