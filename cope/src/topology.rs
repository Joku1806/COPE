use std::vec::Vec;

pub type NodeID = char;

pub struct Topology {
    own_id: NodeID,
    relay: NodeID,
    allowlist: Vec<NodeID>,
}

impl Topology {
    pub fn new(own_id: NodeID, relay: NodeID, allowlist: Vec<NodeID>) -> Topology {
        // FIXME: Exactly how do we pass each node its own ID?
        // There is one global config, so this will be a problem.
        // We also can't use MAC addresses for this. the protocol
        // should not make any assumptions about the running platform.
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
    use crate::topology::Topology;

    #[test]
    fn test_allowlist() {
        let topology: Topology = Topology::new('A', 'B', Vec::from(['B', 'C']));

        assert!(topology.can_receive_from('C'));
        assert!(!topology.can_receive_from('D'));
    }

    #[test]
    fn test_nexthop_outsider() {
        let topology: Topology = Topology::new('A', 'B', Vec::from(['B', 'C']));

        assert_eq!(topology.nexthop_for_target('C'), 'B');
    }

    #[test]
    fn test_nexthop_relay() {
        let topology: Topology = Topology::new('B', 'B', Vec::from(['B', 'C']));

        assert_eq!(topology.nexthop_for_target('C'), 'C');
    }
}
