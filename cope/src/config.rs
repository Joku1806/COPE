// This file is auto generated by a build.rs file and cope_config
use cope_config::config::*;

pub const CONFIG: Config<4> = Config{
    nodes: [
        (NodeID::cnst('A'), MacAdress::cnst([0,0,0,0,0,0,])),
        (NodeID::cnst('B'), MacAdress::cnst([0,0,0,0,0,0,])),
        (NodeID::cnst('C'), MacAdress::cnst([0,0,0,0,0,0,])),
        (NodeID::cnst('D'), MacAdress::cnst([0,0,0,0,0,0,])),
    ],
    relay: NodeID::cnst('B'),
    black_list: [
        (NodeID::cnst('A'), [
            Some(NodeID::cnst('C')),
            None, None, None, 
        ]),
        (NodeID::cnst('B'), [
            None, None, None, None, 
        ]),
        (NodeID::cnst('C'), [
            Some(NodeID::cnst('A')),
            None, None, None, 
        ]),
        (NodeID::cnst('D'), [
            Some(NodeID::cnst('A')),
            Some(NodeID::cnst('B')),
            Some(NodeID::cnst('C')),
            None, 
        ]),
    ]
};