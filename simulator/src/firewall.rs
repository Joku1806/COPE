use crate::esp;
use crate::esp::Packet;

pub struct Firewall {
    pub esp: esp::ESP,
}

impl Firewall {
    pub fn new(esp: esp::ESP) -> Self {
        Firewall { esp }
    }
    pub fn send(&self, packet: Packet) {
        self.esp.send(packet.data);
    }
    pub fn recv(&self) -> Option<Vec<u8>> {
        self.esp.recv()
    }
}
