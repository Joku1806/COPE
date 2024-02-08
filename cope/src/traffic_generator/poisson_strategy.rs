use crate::packet::PacketBuilder;

use rand::prelude::*;
use std::time::{Duration, SystemTime};

use rand_distr;

use super::TGStrategy;

pub struct PoissonStrategy {
    // NOTE: The next timestamp at which to generate a packet
    generation_timestamp: SystemTime,
    // NOTE: The target network throughput in bytes
    generation_rate: f32,
    distribution: rand_distr::Poisson<f32>,
}

impl PoissonStrategy {
    pub fn new(generation_rate: u32) -> Self {
        PoissonStrategy {
            generation_timestamp: SystemTime::now(),
            generation_rate: generation_rate as f32,
            distribution: rand_distr::Poisson::new(generation_rate as f32).unwrap(),
        }
    }
}

impl TGStrategy for PoissonStrategy {
    fn generate(&mut self) -> Option<PacketBuilder> {
        // TODO: We should probably add a check if we are too far away from the generation timestamp.
        // This would indicate that the target generation rate is too high to be achieved by the channel.
        let timestamp = SystemTime::now();
        if timestamp < self.generation_timestamp {
            return None;
        }

        log::debug!(
            "Overshoot by: {:?}",
            timestamp.duration_since(self.generation_timestamp).unwrap()
        );

        // NOTE: In the future, packet size could also be made random
        // using a bimodal distribution, like it is done in the paper.
        const PACKET_SIZE: usize = 128;
        self.generation_rate = self.distribution.sample(&mut rand::thread_rng());
        self.generation_timestamp +=
            Duration::from_secs_f32(PACKET_SIZE as f32 / self.generation_rate);
        Some(PacketBuilder::new().with_data_size(PACKET_SIZE))
    }
}
