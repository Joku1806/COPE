use crate::{
    coding::decode_util::{decode, remove_from_pool},
    packet::{Ack, CodingInfo},
    packet_pool::{PacketPool, SimplePacketPool},
    topology::Topology,
    traffic_generator::TrafficGenerator,
    Packet,
};

use super::{
    decode_util::{ids_for_decoding, is_next_hop},
    CodingError, CodingStrategy,
};

pub struct LeafNodeCoding {
    generator: TrafficGenerator,
    packet_pool: SimplePacketPool,
    acks: Vec<CodingInfo>,
}

impl LeafNodeCoding {
    pub fn new(generator: TrafficGenerator) -> Self {
        Self {
            generator,
            packet_pool: SimplePacketPool::new(8),
            acks: vec![],
        }
    }
}

impl CodingStrategy for LeafNodeCoding {
    fn handle_receive(&mut self, packet: &Packet, topology: &Topology) -> Result<(), CodingError> {
        let is_from_relay = packet.sender() == topology.relay();
        if !is_from_relay {
            //store for coding
            return Ok(());
        }
        // check if node is next_hop for packet
        if !is_next_hop(topology.id(), &packet) {
            log::info!("[Node {}]: Not a next hop of Packet.", topology.id());
            return Ok(());
        }
        // decode
        let (ids, info) = ids_for_decoding(topology.id(), packet, &self.packet_pool)?;
        let decoded_data = decode(&ids, packet.data(), &self.packet_pool);
        log::info!("[Node {}]: Decoded into {:?}", topology.id(), decoded_data);
        remove_from_pool(&mut self.packet_pool, &ids);
        self.acks.push(info);
        return Ok(());
    }

    fn handle_send(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError> {
        if let Some(builder) = self.generator.generate() {
            // FIXME: handle this error
            // TODO: add reception report

            // add acks to header
            let ack = Ack {
                source: topology.id(),
                packets: std::mem::take(&mut self.acks),
            };

            let packet = builder.ack_header(vec![ack]).build().unwrap();
            // FIXME: There is the possible Issue that the packet is never send, in this case we
            // should not save it in packet_pool
            self.packet_pool.push_packet(packet.clone());
            return Ok(Some(packet));
        }
        Ok(None)
    }
}
