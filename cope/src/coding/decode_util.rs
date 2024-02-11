use crate::{
    packet::{CodingInfo, PacketData},
    packet_pool::PacketPool,
};
use cope_config::types::node_id::NodeID;

use super::CodingError;

pub fn is_next_hop(id: NodeID, infos: &[CodingInfo]) -> bool {
    infos.iter().find(|&x| x.nexthop == id).is_some()
}

pub fn ids_for_decoding<PP: PacketPool>(
    id: NodeID,
    infos: &[CodingInfo],
    pool: &PP,
) -> Result<(Vec<usize>, CodingInfo), CodingError> {
    // collect Ids to decode package
    let mut packet_indices: Vec<usize> = vec![];
    let mut packet_info: Option<CodingInfo> = None;
    for info in infos {
        if info.nexthop == id {
            packet_info = Some(info.clone());
            continue;
        }

        let Some(index) = pool.position(&info) else {
            return Err(CodingError::DecodeError(format!(
                "Packet with info {} is needed but was not found",
                &info
            )));
        };

        packet_indices.push(index);
    }

    if packet_indices.len() != infos.len() - 1 {
        return Err(CodingError::DecodeError(format!(
            "Needed {}, but found {} packets.",
            infos.len() - 1,
            packet_indices.len()
        )));
    }

    let Some(packet_info) = packet_info else {
        return Err(CodingError::DecodeError(
            "Packet doesn't contain this Node as Nexthop".into(),
        ));
    };

    Ok((packet_indices, packet_info))
}

// TODO: This could be a member of PacketPool
pub fn remove_from_pool<PP: PacketPool>(pool: &mut PP, packet_indices: &Vec<usize>) {
    for &index in packet_indices {
        pool.remove(index).unwrap();
    }
}

pub fn decode<PP: PacketPool>(
    packet_indices: &Vec<usize>,
    packet_data: &PacketData,
    pool: &PP,
) -> PacketData {
    let mut data: PacketData = packet_data.clone();

    for &index in packet_indices {
        let (_, d) = pool.get_ref(index).unwrap();
        data = data.xor(d);
    }
    return data;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        packet::{CodingHeader as CH, Packet, PacketBuilder},
        packet_pool::SimplePacketPool,
    };

    #[test]
    fn test_ids_for_decoding() {
        let state = TestState::simple();
        let mut pool_node_a = SimplePacketPool::new(8);
        let mut pool_node_c = SimplePacketPool::new(8);

        let CH::Encoded(infos) = state.p2.coding_header() else {
            panic!()
        };

        // Can not decode because packet pool does not contain packets needed
        assert!(ids_for_decoding(state.node_a, &infos, &mut pool_node_a).is_err());
        assert!(ids_for_decoding(state.node_c, &infos, &mut pool_node_c).is_err());

        pool_node_a.push_packet(state.p0.clone());
        pool_node_c.push_packet(state.p1.clone());
        let res0 = ids_for_decoding(state.node_a, &infos, &mut pool_node_a);
        let res1 = ids_for_decoding(state.node_c, &infos, &mut pool_node_c);

        // Now can decode
        assert!(res0.is_ok());
        assert!(res1.is_ok());

        let CH::Native(info0) = state.p0.coding_header() else {
            panic!()
        };
        let CH::Native(info1) = state.p1.coding_header() else {
            panic!()
        };
        assert_eq!(*info1, res0.unwrap().1);
        assert_eq!(*info0, res1.unwrap().1);
    }

    struct TestState {
        node_a: NodeID,
        _node_b: NodeID,
        node_c: NodeID,

        p0: Packet,
        p1: Packet,
        p2: Packet,
    }

    impl TestState {
        fn simple() -> Self {
            let node_a = NodeID::new('A');
            let node_b = NodeID::new('B');
            let node_c = NodeID::new('C');

            let data0 = PacketData::new(vec![0, 0]);
            let data1 = PacketData::new(vec![0, 1]);
            let data2 = PacketData::new(vec![1, 0]);

            let coding_info0 = CodingInfo {
                source: node_a,
                id: 0,
                nexthop: node_c,
            };
            let coding_info1 = CodingInfo {
                source: node_c,
                id: 0,
                nexthop: node_a,
            };

            let p0 = PacketBuilder::new()
                .sender(node_a)
                .data(data0)
                .native_header(coding_info0.clone())
                .ack_header(vec![])
                .build()
                .unwrap();
            let p1 = PacketBuilder::new()
                .sender(node_c)
                .data(data1)
                .native_header(coding_info1.clone())
                .ack_header(vec![])
                .build()
                .unwrap();
            let coding_header = vec![coding_info0, coding_info1];
            let p2 = PacketBuilder::new()
                .sender(node_b)
                .data(data2)
                .encoded_header(coding_header)
                .ack_header(vec![])
                .build()
                .unwrap();

            Self {
                node_a,
                _node_b: node_b,
                node_c,
                p0,
                p1,
                p2,
            }
        }
    }
}
