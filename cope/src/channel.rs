use crate::packet::Packet;

pub trait Channel: Send + Sync {
    fn transmit(&self, packet: &Packet);
    fn receive(&self) -> Option<Packet>;
}
