use crate::esp::Packet;
use crate::firewall;

type Address = [u8; 4];
pub struct Node {
    firewall: firewall::Firewall,
    address: Address,
}
impl Node {
    pub fn new(firewall: firewall::Firewall, address: Address) -> Self {
        Node {
            firewall,
            address,
        }
    }
    pub fn send(&self, packet: Packet) {
        self.firewall.send(packet);
    }
    pub fn recv(&self) -> Option<Vec<u8>> {
        self.firewall.recv()
    }
}
