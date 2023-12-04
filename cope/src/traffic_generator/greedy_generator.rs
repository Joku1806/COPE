use cope_config::types::node_id::NodeID;

use crate::packet::{Packet, PacketID};
use crate::traffic_generator::TrafficGenerator;

use rand::prelude::*;

pub struct GreedyGenerator {
    current_packet_id: PacketID,
    valid_receivers: Vec<NodeID>,
}

// NOTE: A generator that will always return a packet.
// Useful for measuring maximum network throughput.
impl GreedyGenerator {
    pub fn new(valid_receivers: Vec<NodeID>) -> Self {
        GreedyGenerator {
            current_packet_id: 0,
            valid_receivers,
        }
    }
}

impl TrafficGenerator for GreedyGenerator {
    fn generate(&mut self) -> Option<Packet> {
        let mut p = Packet::with_serialized_size(256).unwrap();

        let receiver = self
            .valid_receivers
            .choose(&mut rand::thread_rng())
            .expect("valid_receivers must not be empty");
        p.set_id(self.current_packet_id);
        p.set_receiver(*receiver);
        self.current_packet_id += 1;

        return Some(p);
    }
}
