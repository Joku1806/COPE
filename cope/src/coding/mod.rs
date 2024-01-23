pub mod leaf_node_coding;
pub mod relay_node_coding;
mod decode_util;

use core::fmt;

use crate::topology::Topology;

use super::Packet;

pub trait CodingStrategy {
    fn handle_receive(&mut self, packet: &Packet, topology: &Topology) -> Result<(), CodingError>;
    // this name is bad, but so am I
    fn handle_send(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError>;
}


#[derive(Debug, Clone)]
pub enum CodingError {
    DecodeError(String),
}


impl fmt::Display for CodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO: Write Error Messages!")
    }
}

