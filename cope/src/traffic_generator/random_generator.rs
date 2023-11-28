use crate::packet::Packet;
use crate::traffic_generator::TrafficGenerator;

use rand::prelude::*;
use std::time::SystemTime;

pub struct RandomGenerator {
    // NOTE: The last timestamp at which a packet was generated
    generation_timestamp: SystemTime,
    // NOTE: The target network throughput in bytes
    generation_rate: f32,
    rng: rand::rngs::ThreadRng,
}

// TODO: way to set packet size
impl RandomGenerator {
    pub fn new(generation_rate: f32) -> Self {
        RandomGenerator {
            generation_timestamp: SystemTime::now(),
            generation_rate,
            rng: rand::thread_rng(),
        }
    }
}

impl TrafficGenerator for RandomGenerator {
    fn generate(&mut self) -> Option<Packet> {
        let gen: bool = self.rng.gen_bool(0.5);

        if !gen {
            return None;
        }

        // TODO: Better error handling
        let elapsed = self.generation_timestamp.elapsed().unwrap();
        let target_size = (self.generation_rate * elapsed.as_secs_f32()).floor();
        self.generation_timestamp = SystemTime::now();
        return Some(Packet::with_serialized_size(target_size as usize).unwrap());
    }
}
