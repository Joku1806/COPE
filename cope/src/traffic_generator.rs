use crate::packet::Packet;

// TODO
pub struct TrafficGenerator {}

impl TrafficGenerator {
    pub fn new() -> Self {
        TrafficGenerator {}
    }

    pub fn generate(&self) -> Option<Packet> {
        return Some(Packet::empty());
    }
}
