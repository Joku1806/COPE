use crate::packet::PacketBuilder;
use crate::traffic_generator::TGStrategy;

pub struct GreedyStrategy {}

// NOTE: A generator that will always return a packet.
// Useful for measuring maximum network throughput.
impl GreedyStrategy {
    pub fn new() -> Self {
        GreedyStrategy {}
    }
}

impl TGStrategy for GreedyStrategy {
    fn generate(&mut self) -> Option<PacketBuilder> {
        const PACKET_SIZE: usize = 128;
        Some(PacketBuilder::new().with_data_size(PACKET_SIZE))
    }
}
