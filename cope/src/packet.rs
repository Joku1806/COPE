use bitvec::prelude as bv;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

use cope_config::types::node_id::NodeID;

pub type PacketID = u16;

#[derive(Debug)]
pub enum PacketError {
    InvalidSize,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CodingInfo {
    packet_hash: u32,
    nexthop: NodeID,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ReceptionReport {
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
    fn sender(&self) -> NodeID { self.sender }

    //     pub fn with_serialized_size(size: usize) -> Result<Packet, PacketError> {
    //         let mut packet = Packet::default();

    //         let Ok(base_size) = bincode::serialized_size(&packet) else {
    //             return Err(PacketError::InvalidSize);
    //         };
    //         let Some(rest_size) = size.checked_sub(base_size as usize) else {
    //             return Err(PacketError::InvalidSize);
    //         };

    //         packet.data = vec![0; rest_size];
    //         Ok(packet)
    //     }

    pub fn deserialize_from(bytes: &[u8]) -> Result<Packet, bincode::Error> {
        bincode::deserialize(bytes)
    }

    pub fn serialize_into(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
}

#[derive(Default)]
pub struct PacketBuilder {
    id: PacketID,
    sender: NodeID,
    receiver: NodeID,
    coding_header: Vec<CodingInfo>,
    reception_header: Vec<ReceptionReport>,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {}

impl PacketBuilder {
    pub fn new() -> Self {
        PacketBuilder {
            ..Default::default()
        }
    }

    pub fn id(mut self, id: PacketID) -> Self {
        self.id = id;
        self
    }

    pub fn sender(mut self, sender_id: NodeID) -> Self {
        self.sender = sender_id;
        self
    }

    pub fn receiver(mut self, receiver_id: NodeID) -> Self {
        self.receiver = receiver_id;
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn with_data_size(mut self, data_size: usize) -> Self {
        self.data = vec![0; data_size];
        self
    }

    pub fn build(self) -> Result<Packet, Error> {
        // check if everything is set
        // build
        Ok(Packet {
            id: self.id,
            sender: self.sender,
            receiver: self.receiver,
            coding_header: self.coding_header,
            reception_header: self.reception_header,
            data: self.data,
        })
    }
}
