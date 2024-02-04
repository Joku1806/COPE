use cope_config::types::node_id::NodeID;
use serde::{Deserialize, Serialize};
use super::CodingInfo;



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ack {
    pub source: NodeID,
    pub packets: Vec<CodingInfo>,
}

impl Ack {
    pub fn source(&self) -> NodeID {
        self.source
    }

    pub fn packets(&self) -> &[CodingInfo] {
        self.packets.as_ref()
    }
}
