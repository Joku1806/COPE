use super::TGStrategy;
use crate::packet::PacketBuilder;
use rand::prelude::*;
use rand_distr::Uniform;
use std::time::{Duration, SystemTime};

const MIN_TARGET_SIZE: usize = 32;
const MAX_TARGET_SIZE: usize = 128;
pub struct RandomStrategy {
    // NOTE: The last timestamp at which a packet was generated
    generation_timestamp: SystemTime,
    // NOTE: The target network throughput in bytes
    generation_rate: f32,
}

impl RandomStrategy {
    pub fn new(generation_rate: u32) -> Self {
        RandomStrategy {
            generation_timestamp: SystemTime::now(),
            generation_rate: generation_rate as f32,
        }
    }
}

impl TGStrategy for RandomStrategy {
    fn generate(&mut self) -> Option<PacketBuilder> {
        let timestamp = SystemTime::now();
        if timestamp < self.generation_timestamp {
            return None;
        }

        let mut rng = rand::thread_rng();
        let range = Uniform::from(MIN_TARGET_SIZE..MAX_TARGET_SIZE);
        let target_size = range.sample(&mut rng);

        let send_time = (target_size as f32) / self.generation_rate;
        let send_time_micros: u64 = (send_time * 1000_000f32).floor() as u64;
        self.generation_timestamp = SystemTime::now() + Duration::from_micros(send_time_micros);
        Some(PacketBuilder::new().with_data_size(target_size))
    }
}
