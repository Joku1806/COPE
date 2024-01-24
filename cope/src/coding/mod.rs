pub mod leaf_node_coding;
pub mod retrans_queue;
pub mod relay_node_coding;
mod decode_util;

use core::fmt;

use crate::topology::Topology;
use std::time::Duration;
use super::Packet;

pub const QUEUE_SIZE: usize = 8;
pub const RETRANS_DURATION: Duration = Duration::from_millis(800);

pub trait CodingStrategy {
    fn handle_receive(&mut self, packet: &Packet, topology: &Topology) -> Result<(), CodingError>;
    // this name is bad, but so am I
    fn handle_send(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError>;
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

