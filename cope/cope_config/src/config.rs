use crate::node_id::NodeID;
use macaddr::MacAddr6;

pub type MacAddress = MacAddr6;

trait CopeConfig {}

#[derive(Debug)]
pub struct TmpConfig {
    node_count: usize,
    nodes: Vec<(NodeID, MacAddress)>,
    relay: NodeID,
    black_list: Vec<(NodeID, Vec<NodeID>)>,
}

impl TmpConfig {
    pub fn new(
        nodes: Vec<(NodeID, MacAddress)>,
        relay: NodeID,
        black_list: Vec<(NodeID, Vec<NodeID>)>,
    ) -> Self {
        Self {
            node_count: nodes.len(),
            nodes,
            relay,
            black_list,
        }
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }
    pub fn nodes(&self) -> &Vec<(NodeID, MacAddress)> {
        &self.nodes
    }
    pub fn relay(&self) -> NodeID {
        self.relay
    }
    pub fn black_list(&self) -> &Vec<(NodeID, Vec<NodeID>)> {
        &self.black_list
    }
}

impl CopeConfig for TmpConfig {}

pub struct Config<const N: usize> {
    pub nodes: [(NodeID, MacAddress); N],
    pub relay: NodeID,
    // we technically only need N-1 nodes here but yeah
    pub black_list: [(NodeID, [Option<NodeID>; N]); N],
}

impl<const N: usize> CopeConfig for Config<N> {}
