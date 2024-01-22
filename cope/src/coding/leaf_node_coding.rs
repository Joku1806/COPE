use cope_config::types::node_id::NodeID;

use crate::{
    packet::PacketData,
    packet_pool::{PacketPool, SimplePacketPool},
    topology::Topology,
    traffic_generator::TrafficGenerator,
    Packet,
};

use super::{CodingError, CodingStrategy};

pub struct LeafNodeCoding {
    generator: TrafficGenerator,
    packet_pool: SimplePacketPool,
}

impl LeafNodeCoding {
    pub fn new(generator: TrafficGenerator) -> Self {
        Self {
            generator,
            packet_pool: SimplePacketPool::new(8),
        }
    }
}

impl CodingStrategy for LeafNodeCoding {
    fn handle_receive(&mut self, packet: &Packet, topology: &Topology) -> Result<(), CodingError> {
        let is_from_relay = packet.sender() == topology.relay();
        if is_from_relay {
            // decode
            if !is_next_hop(topology.id(), &packet) {
                log::info!("[Node {}]: Not a next hop of Packet.", topology.id());
            } else if let Some(data) = decode_packet(topology.id(), &packet, &mut self.packet_pool)
            {
                log::info!("[Node {}]: Decoded Packet to {:?}.", topology.id(), data);
            } else {
                log::info!("[Node {}]: Could not decode Packet.", topology.id());
            }
        } else {
            //store for coding
        }
        return Ok(());
    }

    fn handle_send(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError> {
        if let Some(builder) = self.generator.generate() {
            // FIXME: handle this error
            // TODO: add reception report
            let packet = builder.build().unwrap();
            // TODO: There is the possible Issue that the packet is never send, in this case we
            // should not save it in packet_pool
            self.packet_pool.push_packet(packet.clone());
            return Ok(Some(packet));
        }
        Ok(None)
    }
}
// Don't know were to put these Functions
fn is_next_hop(id: NodeID, packet: &Packet) -> bool {
    packet
        .coding_header()
        .iter()
        .find(|&x| x.nexthop == id)
        .is_some()
}

// FIXME: Refactor this mess of a function
fn decode_packet<T: PacketPool>(id: NodeID, packet: &Packet, pool: &mut T) -> Option<PacketData> {
    let mut packet_indices: Vec<usize> = vec![];
    for info in packet.coding_header() {
        let Some(index) = pool.position(&info) else {
            if info.nexthop == id {
                continue;
            }
            return None;
        };
        packet_indices.push(index);
    }
    if packet_indices.len() != packet.coding_header().len() - 1 {
        return None;
    }

    let mut data: PacketData = packet.data().clone();

    for &index in &packet_indices {
        let (_, d) = pool.remove(index).unwrap();
        data = data.xor(&d);
    }
    return Some(data);
}
