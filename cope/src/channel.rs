use crate::packet::Packet;

pub trait Channel {
    fn transmit(&self, packet: &Packet);
    fn receive(&mut self) -> Option<Packet>;
}
