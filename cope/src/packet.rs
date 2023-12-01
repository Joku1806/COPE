use bincode::Error;
use bitvec::prelude as bv;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

// TODO: NodeID should be moved to a different file,
// once we create one. Maybe protocol.rs?
pub type NodeID = char;
pub type PacketID = u16;

#[derive(Debug)]
pub enum PacketError {
    InvalidSize,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct CodingInfo {
    packet_hash: u32,
    nexthop: NodeID,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct ReceptionReport {
    source: NodeID,
    last_id: PacketID,
    preceding_ids: bv::BitVec,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct Packet {
    id: PacketID,
    sender: NodeID,
    receiver: NodeID,
    // NOTE: These could also be HashMaps for easy access.
    // But I am not sure if/when this is needed,
    // so lets stay close to the definition in the paper.
    coding_header: Vec<CodingInfo>,
    reception_header: Vec<ReceptionReport>,
    data: Vec<u8>,
}

impl Packet {
    pub fn new(id: PacketID, sender: NodeID, receiver: NodeID) -> Packet {
        let mut p: Packet = Packet::default();

        p.id = id;
        p.sender = sender;
        p.receiver = receiver;
        p.coding_header = Vec::<CodingInfo>::new();
        p.reception_header = Vec::<ReceptionReport>::new();
        p.data = Vec::<u8>::new();

        return p;
    }

    pub fn empty() -> Packet {
        return Packet::default();
    }

    pub fn with_serialized_size(size: usize) -> Result<Packet, PacketError> {
        let mut packet = Packet::default();

        let Ok(base_size) = bincode::serialized_size(&packet) else {
            return Err(PacketError::InvalidSize);
        };
        let Some(rest_size) = size.checked_sub(base_size as usize) else {
            return Err(PacketError::InvalidSize);
        };

        packet.data = vec![0; rest_size];
        Ok(packet)
    }

    pub fn deserialize_from(bytes: &[u8]) -> Result<Packet, Error> {
        bincode::deserialize(bytes)
    }

    pub fn serialize_into(&self) -> Result<Vec<u8>, Error> {
        bincode::serialize(self)
    }

    pub fn set_sender(&mut self, sender: NodeID) {
        self.sender = sender;
    }

    pub fn get_sender(&self) -> NodeID {
        self.sender
    }

    pub fn set_id(&mut self, id: PacketID) {
        self.id = id;
    }
}
