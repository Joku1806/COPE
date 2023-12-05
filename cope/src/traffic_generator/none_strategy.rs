use super::TGStrategy;
use crate::packet::PacketBuilder;

pub struct NoneStrategy {}

// NOTE: A generator that will always return None.
// Helpful for the relay node in the Alice & Bob example,
// which does not generate any traffic.
impl NoneStrategy {
    pub fn new() -> Self {
        NoneStrategy {}
    }
}

impl TGStrategy for NoneStrategy {
    fn generate(&mut self) -> Option<PacketBuilder> {
        None
    }
}
