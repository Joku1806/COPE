// This file is auto generated by a build.rs file and cope_config
use cope_config::config::*;
use cope_config::types::node_id::NodeID;
use cope_config::types::mac_address::MacAddress;
use cope_config::types::traffic_generator_type::TrafficGeneratorType;

pub const CONFIG: Config<3> = Config{
    nodes: [
        (NodeID::new('A'), MacAddress::new(0, 0, 0, 0, 0, 0)),
        (NodeID::new('B'), MacAddress::new(0, 0, 0, 0, 0, 0)),
        (NodeID::new('C'), MacAddress::new(0, 0, 0, 0, 0, 0)),
    ],
    relay: NodeID::new('B'),
    rx_whitelist: [
        (NodeID::new('A'), [
            Some(NodeID::new('B')),
            None, None, 
        ]),
        (NodeID::new('B'), [
            Some(NodeID::new('A')),
            Some(NodeID::new('C')),
            None, 
        ]),
        (NodeID::new('C'), [
            Some(NodeID::new('B')),
            None, None, 
        ]),
    ],
    tx_whitelist: [
        (NodeID::new('A'), [
            Some(NodeID::new('C')),
            None, None, 
        ]),
        (NodeID::new('B'), [
            Some(NodeID::new('A')),
            Some(NodeID::new('C')),
            None, 
        ]),
        (NodeID::new('C'), [
            Some(NodeID::new('A')),
            None, None, 
        ]),
    ],
    traffic_generators: [
        (NodeID::new('A'), TrafficGeneratorType::Poisson(4096)),
        (NodeID::new('B'), TrafficGeneratorType::None),
        (NodeID::new('C'), TrafficGeneratorType::Random(8192)),
    ],
};
