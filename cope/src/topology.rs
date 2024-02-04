use cope_config::types::node_id::NodeID;
use std::vec::Vec;

pub struct Topology {
    id: NodeID,
    relay: NodeID,
    rxlist: Vec<NodeID>,
    txlist: Vec<NodeID>,
}

impl Topology {
    pub fn new(id: NodeID, relay: NodeID, rxlist: Vec<NodeID>, txlist: Vec<NodeID>) -> Topology {
        return Topology {
            id,
            relay,
            rxlist,
            txlist,
        };
    }

    pub fn is_relay(&self) -> bool {
        self.id == self.relay
    }

    pub fn relay(&self) -> NodeID {
        self.relay
    }

    pub fn can_receive_from(&self, id: NodeID) -> bool {
        return self.rxlist.contains(&id);
    }

    pub fn can_send_to(&self, id: NodeID) -> bool {
        return self.txlist.contains(&id);
    }

    pub fn nexthop_for_target(&self, id: NodeID) -> NodeID {
        // NOTE: Because we only focus on the star topology, we have two paths:
        // 1. If we are the relay, we know that the next hop will be the target
        // 2. Otherwise, we have to send the packet to the relay
        if self.id == self.relay {
            return id;
        } else {
            return self.relay;
        }
    }

    pub fn id(&self) -> NodeID {
        self.id
    }

    pub fn txlist(&self) -> &[NodeID] {
        self.txlist.as_ref()
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
            vec![NodeID::new('B'), NodeID::new('C')],
            vec![],
        );

        assert!(topology.can_receive_from(NodeID::new('C')));
        assert!(!topology.can_receive_from(NodeID::new('D')));
    }

    #[test]
    fn test_nexthop_outsider() {
        let topology: Topology = Topology::new(
            NodeID::new('A'),
            NodeID::new('B'),
            vec![NodeID::new('B'), NodeID::new('C')],
            vec![],
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
            vec![NodeID::new('B'), NodeID::new('C')],
            vec![],
        );

        assert_eq!(
            topology.nexthop_for_target(NodeID::new('C')),
            NodeID::new('C')
        );
    }
}
