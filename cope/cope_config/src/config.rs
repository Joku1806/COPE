use crate::types::mac_address::MacAddress;
use crate::types::node_id::NodeID;
use crate::types::traffic_generator_type::TrafficGeneratorType;

trait CopeConfig {}

#[derive(Debug)]
pub struct TmpConfig {
    node_count: usize,
    nodes: Vec<(NodeID, MacAddress)>,
    relay: NodeID,
    rx_whitelist: Vec<(NodeID, Vec<NodeID>)>,
    tx_whitelist: Vec<(NodeID, Vec<NodeID>)>,
    traffic_generators: Vec<(NodeID, TrafficGeneratorType)>,
}

impl TmpConfig {
    pub fn new(
        nodes: Vec<(NodeID, MacAddress)>,
        relay: NodeID,
        rx_whitelist: Vec<(NodeID, Vec<NodeID>)>,
        tx_whitelist: Vec<(NodeID, Vec<NodeID>)>,
        traffic_generators: Vec<(NodeID, TrafficGeneratorType)>,
    ) -> Self {
        Self {
            node_count: nodes.len(),
            nodes,
            relay,
            rx_whitelist,
            tx_whitelist,
            traffic_generators,
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

    pub fn rx_whitelist(&self) -> &Vec<(NodeID, Vec<NodeID>)> {
        &self.rx_whitelist
    }

    pub fn tx_whitelist(&self) -> &Vec<(NodeID, Vec<NodeID>)> {
        &self.tx_whitelist
    }

    pub fn traffic_generators(&self) -> &Vec<(NodeID, TrafficGeneratorType)> {
        &self.traffic_generators
    }
}

impl CopeConfig for TmpConfig {}

pub struct Config<const N: usize> {
    pub nodes: [(NodeID, MacAddress); N],
    pub relay: NodeID,
    // we technically only need N-1 nodes here but yeah
    pub rx_whitelist: [(NodeID, [Option<NodeID>; N]); N],
    pub tx_whitelist: [(NodeID, [Option<NodeID>; N]); N],
    pub traffic_generators: [(NodeID, TrafficGeneratorType); N],
}

impl<const N: usize> CopeConfig for Config<N> {}
