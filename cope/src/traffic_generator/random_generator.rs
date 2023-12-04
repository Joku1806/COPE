use crate::packet::{Packet, PacketID};
use crate::traffic_generator::TrafficGenerator;

use cope_config::types::node_id::NodeID;
use rand::prelude::*;
use std::time::SystemTime;

pub struct RandomGenerator {
    // NOTE: The last timestamp at which a packet was generated
    generation_timestamp: SystemTime,
    // NOTE: The target network throughput in bytes
    generation_rate: f32,
    current_packet_id: PacketID,
    valid_receivers: Vec<NodeID>,
}

// TODO: way to set packet size
impl RandomGenerator {
    pub fn new(generation_rate: f32, valid_receivers: Vec<NodeID>) -> Self {
        RandomGenerator {
            generation_timestamp: SystemTime::now(),
            generation_rate,
            current_packet_id: 0,
            valid_receivers,
        }
    }
}

impl TrafficGenerator for RandomGenerator {
    fn generate(&mut self) -> Option<Packet> {
        let gen: bool = rand::thread_rng().gen_bool(0.5);

        if !gen {
            return None;
        }

        // FIXME: do a functioning target_size
        // TODO: Better error handling
        // let elapsed = self.generation_timestamp.elapsed().unwrap();
        // let target_size = (self.generation_rate * elapsed.as_secs_f32()).floor();
        // self.generation_timestamp = SystemTime::now();
        let target_size = 200;
        let receiver = self
            .valid_receivers
            .choose(&mut rand::thread_rng())
            .expect("valid_receivers must not be empty");

        let mut p = Packet::with_serialized_size(target_size as usize).unwrap();
        p.set_id(self.current_packet_id);
        p.set_receiver(*receiver);

        self.current_packet_id = self.current_packet_id.checked_add(1).unwrap_or(0);

        return Some(p);
    }
}
