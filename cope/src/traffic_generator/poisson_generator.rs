use crate::packet::{Packet, PacketID};
use crate::traffic_generator::TrafficGenerator;

use cope_config::types::node_id::NodeID;
use rand::prelude::*;
use std::time::{Duration, SystemTime};

use rand_distr;

pub struct PoissonGenerator {
    // NOTE: The next timestamp at which to generate a packet
    generation_timestamp: SystemTime,
    // NOTE: The target network throughput in bytes
    generation_rate: f32,
    distribution: rand_distr::Poisson<f32>,
    current_packet_id: PacketID,
    valid_receivers: Vec<NodeID>,
}

// TODO: way to set packet size
impl PoissonGenerator {
    pub fn new(generation_rate: u32, valid_receivers: Vec<NodeID>) -> Self {
        PoissonGenerator {
            generation_timestamp: SystemTime::now(),
            generation_rate: generation_rate as f32,
            distribution: rand_distr::Poisson::new(generation_rate as f32).unwrap(),
            current_packet_id: 0,
            valid_receivers,
        }
    }
}

impl TrafficGenerator for PoissonGenerator {
    fn generate(&mut self) -> Option<Packet> {
        // TODO: We should probably add a check if we are too far away from the generation timestamp.
        // This would indicate that the target generation rate is too high to be achieved by the channel.
        if SystemTime::now() < self.generation_timestamp {
            return None;
        }

        // NOTE: In the future, packet size could also be made random
        // using a bimodal distribution, like it is done in the paper.
        const PACKET_SIZE: usize = 250;
        // TODO: Better error handling
        let receiver = self
            .valid_receivers
            .choose(&mut rand::thread_rng())
            .expect("valid_receivers must not be empty");
        let mut p = Packet::with_serialized_size(PACKET_SIZE).unwrap();
        p.set_id(self.current_packet_id);
        p.set_receiver(*receiver);

        self.current_packet_id += 1;
        self.generation_rate = self.distribution.sample(&mut rand::thread_rng());
        self.generation_timestamp +=
            Duration::from_secs_f32(PACKET_SIZE as f32 / self.generation_rate);

        return Some(p);
    }
}
