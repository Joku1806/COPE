use cope_config::types::node_id::NodeID;
use std::vec::Vec;

pub struct Topology {
    own_id: NodeID,
    relay: NodeID,
    allowlist: Vec<NodeID>,
}

impl Topology {
    pub fn new(own_id: NodeID, relay: NodeID, allowlist: Vec<NodeID>) -> Topology {
        return Topology {
            own_id,
            relay,
            allowlist,
        };
    }

    pub fn can_receive_from(&self, id: NodeID) -> bool {
        return self.allowlist.contains(&id);
    }

    pub fn nexthop_for_target(&self, id: NodeID) -> NodeID {
        // NOTE: Because we only focus on the star topology, we have two paths:
        // 1. If we are the relay, we know that the next hop will be the target
        // 2. Otherwise, we have to send the packet to the relay
        if self.own_id == self.relay {
            return id;
        } else {
            return self.relay;
        }
    }
}

#[cfg(test)]
mod test {
    use cope_config::types::node_id::NodeID;

    use crate::topology::Topology;

    #[test]
    fn test_allowlist() {
        let topology: Topology = Topology::new(
            NodeID::new('A'),
            NodeID::new('B'),
            Vec::from([NodeID::new('B'), NodeID::new('C')]),
        );

        assert!(topology.can_receive_from(NodeID::new('C')));
        assert!(!topology.can_receive_from(NodeID::new('D')));
    }

    #[test]
    fn test_nexthop_outsider() {
        let topology: Topology = Topology::new(
            NodeID::new('A'),
            NodeID::new('B'),
            Vec::from([NodeID::new('B'), NodeID::new('C')]),
        );

        assert_eq!(
            topology.nexthop_for_target(NodeID::new('C')),
            NodeID::new('B')
        );
    }

    #[test]
    fn test_nexthop_relay() {
        let topology: Topology = Topology::new(
            NodeID::new('B'),
            NodeID::new('B'),
            Vec::from([NodeID::new('B'), NodeID::new('C')]),
        );

        assert_eq!(
            topology.nexthop_for_target(NodeID::new('C')),
            NodeID::new('C')
        );
    }
}
