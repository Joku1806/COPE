use crate::packet::Packet;
use crate::traffic_generator::TrafficGenerator;

pub struct NoneGenerator {}

// NOTE: A generator that will always return None.
// Helpful for the relay node in the Alice & Bob example,
// which does not generate any traffic.
impl NoneGenerator {
    pub fn new() -> Self {
        NoneGenerator {}
    }
}

impl TrafficGenerator for NoneGenerator {
    fn generate(&mut self) -> Option<Packet> {
        None
    }
}
