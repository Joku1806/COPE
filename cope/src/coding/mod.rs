mod decode_util;
pub mod leaf_node_coding;
pub mod relay_node_coding;
pub mod retrans_queue;

use core::fmt;

use super::Packet;
use crate::{packet::PacketData, topology::Topology};

pub const MAX_RETRANS_AMOUNT: u8 = 2;

pub trait CodingStrategy {
    fn handle_rx(
        &mut self,
        packet: &Packet,
        topology: &Topology,
    ) -> Result<Option<PacketData>, CodingError>;
    fn handle_tx(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError>;
    fn update_last_packet_send(&mut self);
}

#[derive(Debug, Clone)]
pub enum CodingError {
    DecodeError(String),
    DefectPacketError(String),
    FullRetransQueue(String),
}

impl fmt::Display for CodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DecodeError(str) => write!(f, "[DecodeError]: {}", str),
            Self::DefectPacketError(str) => write!(f, "[DefectPacketError]: {}", str),
            Self::FullRetransQueue(str) => write!(f, "[FullRetransQueue]: {}", str),
        }
    }
}
