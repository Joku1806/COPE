use crate::packet::Packet;

pub enum ChannelError {
    ReachedMaxBackoffs,
    NoACK,
    UnknownReceiver,
}

pub trait Channel {
    fn transmit(&self, packet: &Packet) -> Result<(), ChannelError>;
    fn receive(&mut self) -> Option<Packet>;
}
