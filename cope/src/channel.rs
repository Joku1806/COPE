use crate::packet::Packet;
use std::error::Error;

pub trait Channel {
    fn transmit(&mut self, packet: &Packet) -> Result<(), Box<dyn Error>>;
    fn receive(&mut self) -> Option<Packet>;
}
