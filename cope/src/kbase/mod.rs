use crate::packet::CodingInfo;
use cope_config::types::node_id::NodeID;

pub mod simple_kbase;

pub use simple_kbase::SimpleKBase;

pub trait KBase {
    fn knows(&self, next_hop: &NodeID, info: &CodingInfo) -> bool;
    fn insert(&mut self, next_hop: NodeID, info: CodingInfo);
    fn size(&self) -> usize;
}
