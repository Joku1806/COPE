use bitvec::prelude as bv;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::vec::Vec;

use cope_config::types::node_id::NodeID;

use super::Ack;
use super::PacketData;

pub type PacketID = u16;

#[derive(Debug)]
pub enum PacketError {
    InvalidSize,
}

#[derive(Debug)]
pub enum PacketReceiver {
    Single(NodeID),
    Multi,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CodingHeader {
    Native(CodingInfo),
    Encoded(Vec<CodingInfo>),
    Control(NodeID),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    sender: NodeID,
    coding_header: CodingHeader,
    reception_header: Vec<ReceptionReport>,
    ack_header: Vec<Ack>,
    data: PacketData,
}

impl Packet {
    pub fn sender(&self) -> NodeID {
        self.sender
    }

    pub fn data(&self) -> &PacketData {
        &self.data
    }

    pub fn coding_header(&self) -> &CodingHeader {
        &self.coding_header
    }

    pub fn ack_header(&self) -> &[Ack] {
        &self.ack_header
    }

    // FIXME: This is just a hack, we always need `some` NodeID to act as the receiver,
    // because the ESPChannel needs to internally translate the receiver to a single MAC address.
    // Check if we should instead change receiver to what it was before!
    pub fn canonical_receiver(&self) -> Option<NodeID> {
        match self.coding_header {
            CodingHeader::Native(ref info) => Some(info.nexthop),
            CodingHeader::Encoded(ref infos) => Some(infos.first().unwrap().nexthop),
            CodingHeader::Control(node_id) => Some(node_id),
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
    sender: Option<NodeID>,
    coding_header: Option<CodingHeader>,
    reception_header: Option<Vec<ReceptionReport>>,
    ack_header: Option<Vec<Ack>>,
    data: Option<PacketData>,
}

#[derive(Debug)]
pub struct PacketBuildError(&'static str);

impl Display for PacketBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[PacketBuildError]: {}", self.0)
    }
}

impl PacketBuilder {
    pub fn new() -> Self {
        PacketBuilder {
            ..Default::default()
        }
    }

    pub fn sender(mut self, sender_id: NodeID) -> Self {
        self.sender = Some(sender_id);
        self
    }

    pub fn data(mut self, data: PacketData) -> Self {
        self.data = Some(data);
        self
    }

    pub fn data_raw(mut self, data: Vec<u8>) -> Self {
        self.data = Some(PacketData::new(data));
        self
    }

    pub fn with_data_size(mut self, data_size: usize) -> Self {
        self.data = Some(PacketData::new(vec![0; data_size]));
        self
    }

    pub fn control_header(mut self, id: NodeID) -> Self {
        self.coding_header = Some(CodingHeader::Control(id));
        self
    }

    pub fn encoded_header(mut self, infos: Vec<CodingInfo>) -> Self {
        self.coding_header = Some(CodingHeader::Encoded(infos));
        self
    }

    pub fn native_header(mut self, info: CodingInfo) -> Self {
        self.coding_header = Some(CodingHeader::Native(info));
        self
    }

    pub fn ack_header(mut self, ack_header: Vec<Ack>) -> Self {
        self.ack_header = Some(ack_header);
        self
    }

    pub fn build(self) -> Result<Packet, PacketBuildError> {
        // check if everything is set correctly
        let Some(sender) = self.sender else {
            return Err(PacketBuildError("Sender must be specified"));
        };

        let Some(coding_header) = self.coding_header else {
            return Err(PacketBuildError("Coding Header must be specified."));
        };

        // TODO: add ReceptionReport
        // let Some(reception_header) = self.reception_header else {
        //     return Err(PacketBuildError("Reception Report must be specified."));
        // };

        let Some(ack_header) = self.ack_header else {
            return Err(PacketBuildError("Ack Header must be specified."));
        };

        use CodingHeader as CH;
        let data = match (&coding_header, self.data) {
            (CH::Native(_), Some(data)) => data,
            (CH::Native(_), None) => {
                return Err(PacketBuildError("Native Packet must have Packet Data."));
            }
            (CH::Encoded(_), Some(data)) => data,
            (CH::Encoded(_), None) => {
                return Err(PacketBuildError("Encoded Packet must have Packet Data."));
            }
            (CH::Control(_), Some(_)) => {
                return Err(PacketBuildError(
                    "Control Packet cannot contain Packet Data.",
                ));
            }
            (CH::Control(_), None) => PacketData::new(vec![]),
        };
        // build
        Ok(Packet {
            sender,
            coding_header,
            reception_header: vec![],
            ack_header,
            data,
        })
    }
}
