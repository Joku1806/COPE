use std::collections::HashSet;

use super::{PPEntry, PacketPool};
use crate::packet::{CodingInfo, Packet, packet::CodingHeader};
use cope_config::types::node_id::NodeID;

// NOTE: This is the most simple way to implement a packet pool
// It will store at most max_size elements
// If it tries to store more it replaces the oldes element
// We could use a ring buffer but this would be more complex
// Using this setup, the relay will also forget
// GC does nothing
pub struct SimplePacketPool {
    queue: Vec<PPEntry>,
    max_size: usize,
}

impl SimplePacketPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Vec::new(),
            max_size,
        }
    }

    pub fn unique_nexthops(&self) -> usize {
        let uniques: HashSet<NodeID> = self.queue.iter().map(|(ci, _)| ci.nexthop).collect();
        uniques.len()
    }
}

impl PacketPool for SimplePacketPool {
    fn peek_front(&self) -> Option<&PPEntry> {
        unimplemented!();
    }

    fn pop_front(&mut self) -> Option<PPEntry> {
        if self.queue.first().is_none() {
            return None;
        }
        Some(self.queue.remove(0))
    }

    fn peek_nexthop_front(&self, nexthop: NodeID) -> Option<&PPEntry> {
        self.queue.iter().find(|x| x.0.nexthop == nexthop)
    }

    fn pop_nexthop_front(&mut self, nexthop: NodeID) -> Option<PPEntry> {
        let Some(pos) = self.queue.iter().position(|x| x.0.nexthop == nexthop) else {
            return None;
        };
        Some(self.queue.remove(pos))
    }

    fn get(&self, pos: usize) -> Option<&PPEntry> {
        self.queue.get(pos)
    }

    fn position(&self, info: &CodingInfo) -> Option<usize> {
        self.queue.iter().position(|x| x.0 == *info)
    }

    fn remove(&mut self, pos: usize) -> Option<PPEntry> {
        Some(self.queue.remove(pos))
    }

    fn push_packet(&mut self, packet: Packet) {
        let is_at_max_size = self.queue.len() >= self.max_size;
        if is_at_max_size {
            self.pop_front();
        }
        let CodingHeader::Native(info) = packet.coding_header() else {
            panic!("Expected Native Packet");
        };
        let data = packet.data();
        self.queue.push((info.clone(), data.clone()));
    }

    fn garbage_collect() {}

    fn size(&self) -> usize {
        self.queue.len()
    }

    fn get_ref(&self, pos: usize) -> Option<&PPEntry> {
        self.queue.get(pos)
    }
}
