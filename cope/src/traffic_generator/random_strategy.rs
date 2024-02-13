use super::{data_generator::DataGenerator, size_distribution::SizeDistribution, TGStrategy};
use crate::packet::PacketBuilder;
use std::time::{Duration, SystemTime};

pub struct RandomStrategy {
    // NOTE: The last timestamp at which a packet was generated
    generation_timestamp: SystemTime,
    // NOTE: The target network throughput in bytes
    generation_rate: f32,
    // NOTE: Distribution of packet sizes
    size_distribution: SizeDistribution,
    // NOTE: Data generator
    data_generator: DataGenerator,
}

impl RandomStrategy {
    pub fn new(generation_rate: u32) -> Self {
        RandomStrategy {
            generation_timestamp: SystemTime::now(),
            generation_rate: generation_rate as f32,
            size_distribution: SizeDistribution::new(),
            data_generator: DataGenerator::new(),
        }
    }
}

impl TGStrategy for RandomStrategy {
    fn generate(&mut self) -> Option<PacketBuilder> {
        let timestamp = SystemTime::now();
        if timestamp < self.generation_timestamp {
            return None;
        }

        log::debug!(
            "Overshoot by: {:?}",
            timestamp.duration_since(self.generation_timestamp).unwrap()
        );

        let target_size = self.size_distribution.sample(&mut rand::thread_rng());
        let send_time = (target_size as f32) / self.generation_rate;
        let send_time_micros: u64 = (send_time * 1000_000f32).floor() as u64;
        self.generation_timestamp = SystemTime::now() + Duration::from_micros(send_time_micros);
        Some(PacketBuilder::new().data_raw(self.data_generator.generate(target_size)))
    }
}
