use bitvec::prelude as bv;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

use cope_config::types::node_id::NodeID;

use super::PacketData;

pub type PacketID = u16;

static mut CURRENT_PACKET_ID: PacketID = 0;

#[inline]
pub fn next_packet_id() -> PacketID {
    unsafe {
        CURRENT_PACKET_ID = CURRENT_PACKET_ID.checked_add(1).unwrap_or(0);
        return CURRENT_PACKET_ID;
    }
}

#[derive(Debug)]
pub enum PacketError {
    InvalidSize,
}

#[derive(Debug)]
pub enum PacketReceiver {
    Single(NodeID),
    Multi
}

#[derive(Debug)]
pub enum CodingHeader {
    Native(CodingInfo),
    Encoded(Vec<CodingInfo>),
    ReportOnly
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CodingInfo {
    pub source: NodeID,
    pub id: PacketID,
    pub nexthop: NodeID,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReceptionReport {
    source: NodeID,
    last_id: PacketID,
    preceding_ids: bv::BitVec,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Packet {
    id: PacketID,
    sender: NodeID,
    // NOTE: These could also be HashMaps for easy access.
    // But I am not sure if/when this is needed,
    // so lets stay close to the definition in the paper.
    coding_header: Vec<CodingInfo>,
    reception_header: Vec<ReceptionReport>,
    data: PacketData,
}

impl Packet {
    pub fn sender(&self) -> NodeID { self.sender }
    pub fn receiver(&self) -> PacketReceiver {
        if self.coding_header.len() == 1 {
            let receiver = self.coding_header.first().unwrap().nexthop;
            PacketReceiver::Single(receiver)
        } else {
            PacketReceiver::Multi
        }
    }
    pub fn id(&self) -> PacketID { self.id }
    pub fn data(&self) -> &PacketData { &self.data }
    pub fn coding_header(&self) -> &Vec<CodingInfo> { &self.coding_header }

    // FIXME: This is just a hack, we always need `some` NodeID to act as the receiver,
    // because the ESPChannel needs to internally translate the receiver to a single MAC address.
    // Check if we should instead change receiver to what it was before!
    pub fn canonical_receiver(&self) -> Option<NodeID> {
        match self.coding_header.len() {
            0 => None,
            _ => Some(self.coding_header.first().unwrap().nexthop),
        }
    }

    pub fn set_sender(mut self, sender: NodeID) -> Self {
        self.sender = sender;
        self
    }

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
    data: PacketData,
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
            id: next_packet_id(),
            ..Default::default()
        }
    }

    pub fn id_from(mut self, id: PacketID) -> Self {
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

    pub fn data(mut self, data: PacketData) -> Self {
        self.data = data;
        self
    }

    pub fn data_raw(mut self, data: Vec<u8>) -> Self {
        self.data = PacketData::new(data);
        self
    }

    pub fn with_data_size(mut self, data_size: usize) -> Self {
        self.data = PacketData::new(vec![0; data_size]);
        self
    }

    pub fn coding_header(mut self, coding_header: Vec<CodingInfo>) -> Self {
        self.coding_header = coding_header;
        self
    }


    pub fn single_coding_header(mut self, source: NodeID, nexthop: NodeID) -> Self {
        self.coding_header = vec![CodingInfo {
            source, nexthop, id: self.id
            }];
        self
    }

    pub fn build(self) -> Result<Packet, Error> {
        // check if everything is set
        // build
        Ok(Packet {
            id: self.id,
            sender: self.sender,
            coding_header: self.coding_header,
            reception_header: self.reception_header,
            data: self.data,
        })
    }
}
