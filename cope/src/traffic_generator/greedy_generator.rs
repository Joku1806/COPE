use crate::packet::Packet;
use crate::traffic_generator::TrafficGenerator;

pub struct GreedyGenerator {}

// TODO: way to set packet size
// NOTE: A generator that will always return a packet.
// Useful for measuring maximum network throughput.
impl GreedyGenerator {
    pub fn new() -> Self {
        GreedyGenerator {}
    }
}

impl TrafficGenerator for GreedyGenerator {
    fn generate(&mut self) -> Option<Packet> {
        // TODO: way to set data
        Some(Packet::empty())
    }
}
