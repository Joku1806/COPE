use crate::packet::Packet;

#[derive(Debug)]
pub enum ChannelError {
    ReachedMaxBackoffs,
    NoACK,
}

pub trait Channel {
    fn transmit(&self, packet: &Packet) -> Result<(), ChannelError>;
    fn receive(&mut self) -> Option<Packet>;
}
