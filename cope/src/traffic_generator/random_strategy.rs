use super::TGStrategy;
use crate::packet::PacketBuilder;
use rand::prelude::*;
use std::time::SystemTime;

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
        let gen: bool = rand::thread_rng().gen_bool(0.5);

        if !gen {
            return None;
        }

        // TODO: Better error handling
        let elapsed = self.generation_timestamp.elapsed().unwrap();
        let target_size = (self.generation_rate * elapsed.as_secs_f32()).floor() as usize;
        self.generation_timestamp = SystemTime::now();
        Some(PacketBuilder::new().with_data_size(target_size))
    }
}
