use std::time::Duration;

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
    pub simulator_packet_loss: f64,
    pub round_trip_time: Duration,
    pub packet_pool_size: usize,
    pub control_packet_duration: Duration,
    pub max_retrans_amount: u8,
    pub use_coding: bool,
    pub stats_log_duration: Duration,
    pub log_node_stats: bool,
    pub log_espnow_stats: bool,
}

impl TmpConfig {
    pub fn new(
        nodes: Vec<(NodeID, MacAddress)>,
        relay: NodeID,
        rx_whitelist: Vec<(NodeID, Vec<NodeID>)>,
        tx_whitelist: Vec<(NodeID, Vec<NodeID>)>,
        traffic_generators: Vec<(NodeID, TrafficGeneratorType)>,
        simulator_packet_loss: f64,
        round_trip_time: Duration,
        packet_pool_size: usize,
        control_packet_duration: Duration,
        max_retrans_amount: u8,
        use_coding: bool,
        stats_log_duration: Duration,
        log_node_stats: bool,
        log_espnow_stats: bool,
    ) -> Self {
        Self {
            node_count: nodes.len(),
            nodes,
            relay,
            rx_whitelist,
            tx_whitelist,
            traffic_generators,
            simulator_packet_loss,
            round_trip_time,
            packet_pool_size,
            control_packet_duration,
            max_retrans_amount,
            use_coding,
            stats_log_duration,
            log_node_stats,
            log_espnow_stats,
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
    pub simulator_packet_loss: f64,
    pub round_trip_time: Duration,
    pub control_packet_duration: Duration,
    pub packet_pool_size: usize,
    pub max_retrans_amount: u8,
    pub use_coding: bool,
    pub stats_log_duration: Duration,
    pub log_node_stats: bool,
    pub log_espnow_stats: bool,
}

impl<const N: usize> CopeConfig for Config<N> {}

impl<const N: usize> Config<N> {
    pub fn get_node_ids(&self) -> Vec<NodeID> {
        let mut v = vec![];

        for i in 0..N {
            v.push(self.nodes[i].0);
        }

        v
    }

    pub fn get_node_id_for(&self, mac: MacAddress) -> Option<NodeID> {
        for i in 0..N {
            if self.nodes[i].1 == mac {
                return Some(self.nodes[i].0);
            }
        }

        None
    }

    pub fn get_rx_whitelist_for(&self, id: NodeID) -> Option<Vec<NodeID>> {
        self.rx_whitelist
            .iter()
            .find(|&&(node, _)| id == node)
            .map(|(_, list)| list.iter().filter_map(|opt| *opt).collect::<Vec<NodeID>>())
    }

    pub fn get_tx_whitelist_for(&self, id: NodeID) -> Option<Vec<NodeID>> {
        self.tx_whitelist
            .iter()
            .find(|&&(node, _)| id == node)
            .map(|(_, list)| list.iter().filter_map(|opt| *opt).collect::<Vec<NodeID>>())
    }

    pub fn get_generator_type_for(&self, id: NodeID) -> Option<TrafficGeneratorType> {
        for i in 0..N {
            if self.traffic_generators[i].0 == id {
                return Some(self.traffic_generators[i].1);
            }
        }

        None
    }
}
