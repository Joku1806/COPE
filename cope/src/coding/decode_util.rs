use crate::{
    packet::{CodingInfo, PacketData},
    packet_pool::PacketPool,
    Packet,
};
use cope_config::types::node_id::NodeID;

use super::CodingError;


pub fn is_next_hop(id: NodeID, infos: &[CodingInfo]) -> bool {
    infos
        .iter()
        .find(|&x| x.nexthop == id)
        .is_some()
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
                "Packet with info {:?} is needed but was not found",
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
mod tests {
    use super::*;
    use crate::{packet::PacketBuilder, packet_pool::SimplePacketPool};

    #[test]
    fn test_ids_for_decoding() {
        let state = TestState::simple();
        let mut pool_node_a = SimplePacketPool::new(8);
        let mut pool_node_c = SimplePacketPool::new(8);

        assert!(ids_for_decoding(state.node_a, &state.p2, &mut pool_node_a).is_err());
        assert!(ids_for_decoding(state.node_c, &state.p2, &mut pool_node_c).is_err());

        pool_node_a.push_packet(state.p0.clone());
        pool_node_c.push_packet(state.p1.clone());
        let res0 = ids_for_decoding(state.node_a, &state.p2, &mut pool_node_a);
        let res1 = ids_for_decoding(state.node_c, &state.p2, &mut pool_node_c);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(state.p1.coding_header()[0], res0.unwrap().1);
        assert_eq!(state.p0.coding_header()[0], res1.unwrap().1);
    }

    struct TestState {
        node_a: NodeID,
        node_b: NodeID,
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

            let p0 = PacketBuilder::new()
                .sender(node_a)
                .data(data0)
                .single_coding_header(node_a, node_c)
                .build()
                .unwrap();
            let p1 = PacketBuilder::new()
                .sender(node_c)
                .data(data1)
                .single_coding_header(node_c, node_a)
                .build()
                .unwrap();
            let coding_header = vec![p0.coding_header()[0].clone(), p1.coding_header()[0].clone()];
            let p2 = PacketBuilder::new()
                .sender(node_b)
                .data(data2)
                .coding_header(coding_header)
                .build()
                .unwrap();

            Self {
                node_a,
                node_b,
                node_c,
                p0,
                p1,
                p2,
            }
        }
    }
}
