use crate::packet::{Packet, CodingInfo};
use cope_config::types::node_id::NodeID;
use super::{PacketPool, PPEntry};

// NOTE: This is the most simple way to implement a packet pool
// NOTE: It will store at most max_size elements
// NOTE: If it tries to store more it replaces the oldes element
// NOTE: We could use a ring buffer but this would be more complex
// NOTE: Using this setup, the relay will also forget
// NOTE: GC does nothing
pub struct SimplePacketPool{
    queue: Vec<PPEntry>,
    max_size: usize,
}

impl SimplePacketPool {
    pub fn new(max_size: usize) -> Self{
        Self { queue: Vec::new(), max_size}
    }
}

impl PacketPool for SimplePacketPool {
    fn peek_front(&self) -> Option<&PPEntry> {
        unimplemented!();
    }

    fn pop_front(&mut self) -> Option<PPEntry> {
        if self.queue.first().is_none() { return None; }
        Some(self.queue.remove(0))
    }

    fn peek_nexthop_front(&self, nexthop: NodeID) -> Option<&PPEntry> {
        self.queue.iter().find(|x| x.0.nexthop == nexthop)
    }

    fn pop_nexthop_front(&mut self, nexthop: NodeID) -> Option<PPEntry> {
        let Some(pos) = self.queue.iter().position(|x| x.0.nexthop == nexthop) else { return None };
        Some(self.queue.remove(pos))
    }

    fn position(&mut self, info: &CodingInfo) -> Option<usize> {
        self.queue.iter().position(|x| x.0 == *info)
    }

    fn remove(&mut self, pos: usize) -> Option<PPEntry> {
        Some(self.queue.remove(pos))
    }

    fn push_packet(&mut self, packet: Packet) {
        let is_at_max_size = self.queue.len() >= self.max_size;
        if  is_at_max_size { self.pop_front(); }
        if packet.coding_header().len() != 1 {
            panic!("Expected Native Packet");
        }
        let info = packet.coding_header().first().unwrap();
        let data = packet.data();
        self.queue.push((info.clone(), data.clone()));
    }

    fn garbage_collect() {}

    fn size(&self) -> usize {
        self.queue.len()
    }
}