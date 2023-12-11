pub mod simple_packet_pool;


use crate::packet::PacketData;

use super::packet::{Packet, CodingInfo};
use cope_config::types::node_id::NodeID;
pub use simple_packet_pool::SimplePacketPool;

pub type PPEntry = (CodingInfo, PacketData);

pub trait PacketPool {
    fn peek_front(&self) -> Option<&PPEntry>;
    fn pop_front(&mut self) -> Option<PPEntry>;
    fn peek_nexthop_front(&self, nexthop: NodeID) -> Option<&PPEntry>;
    fn pop_nexthop_front(&mut self, nexthop: NodeID) -> Option<PPEntry>;
    fn get(&self, pos: usize) -> Option<&PPEntry>;
    fn position(&mut self, info: &CodingInfo) -> Option<usize>;
    fn remove(&mut self, pos: usize) -> Option<PPEntry>;
    fn push_packet(&mut self, packet: Packet);
    fn garbage_collect();
    fn size(&self) -> usize;
}

