use crate::{
    coding::decode_util::{decode, remove_from_pool},
    packet::{Ack, CodingInfo, PacketBuilder},
    packet_pool::{PacketPool, SimplePacketPool},
    topology::Topology,
    traffic_generator::TrafficGenerator,
    Packet,
};

use super::{
    decode_util::{ids_for_decoding, is_next_hop},
    retrans_queue::RetransQueue,
    CodingError, CodingStrategy, QUEUE_SIZE, RETRANS_DURATION,
};

pub struct LeafNodeCoding {
    generator: TrafficGenerator,
    packet_pool: SimplePacketPool,
    retrans_queue: RetransQueue,
    acks: Vec<CodingInfo>,
}

impl LeafNodeCoding {
    pub fn new(generator: TrafficGenerator) -> Self {
        Self {
            generator,
            packet_pool: SimplePacketPool::new(QUEUE_SIZE),
            retrans_queue: RetransQueue::new(QUEUE_SIZE, RETRANS_DURATION),
            acks: vec![],
        }
    }
}

impl CodingStrategy for LeafNodeCoding {
    fn handle_rx(&mut self, packet: &Packet, topology: &Topology) -> Result<(), CodingError> {
        let is_from_relay = packet.sender() == topology.relay();
        if !is_from_relay {
            //store for coding
            return Ok(());
        }
        // handle acks
        let acks = packet.ack_header();
        for ack in acks {
            for info in ack.packets() {
                log::info!("[Node {}]: Packet {:?} was acked.", topology.id(), info);
                self.retrans_queue.remove_packet(info);
            }
        }
        log::info!("[Node{}]: Retrans Queue Size {:?}", topology.id(), self.retrans_queue.len());

        // check if node is next_hop for packet
        if !is_next_hop(topology.id(), &packet) {
            log::info!("[Node {}]: Not a next hop of Packet.", topology.id());
            return Ok(());
        }
        // decode
        // TODO: add acks to the thing
        let (ids, info) = ids_for_decoding(topology.id(), packet, &self.packet_pool)?;
        let decoded_data = decode(&ids, packet.data(), &self.packet_pool);
        log::info!("[Node {}]: Decoded into {:?}", topology.id(), decoded_data);
        remove_from_pool(&mut self.packet_pool, &ids);
        self.acks.push(info);
        return Ok(());
    }

    fn handle_tx(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError> {
        if let Some((info, data)) = self.retrans_queue.packet_to_retrans() {
            let builder = PacketBuilder::new()
                .sender(topology.id())
                .data(data)
                .coding_header(vec![info]);
            // TODO: add reception report

            // add acks to header
            let ack = Ack {
                source: topology.id(),
                packets: std::mem::take(&mut self.acks),
            };

            let packet = builder.ack_header(vec![ack]).build().unwrap();
            self.packet_pool.push_packet(packet.clone());

            let Some(info) = packet.coding_header().first() else {
                return Err(CodingError::DefectPacketError(
                    "Packet should have coding info".into(),
                ));
            };
            return Ok(Some(packet));
        }

        if self.retrans_queue.is_full() {
            return Err(CodingError::FullRetransQueue(
                format!("[Node {}]: Cannot send new packet, without dropping old Packet.", topology.id()),
            ));
        }

        if let Some(builder) = self.generator.generate() {
            // TODO: add reception report

            // add acks to header
            let ack = Ack {
                source: topology.id(),
                packets: std::mem::take(&mut self.acks),
            };

            let packet = builder.ack_header(vec![ack]).build().unwrap();
            self.packet_pool.push_packet(packet.clone());

            let Some(info) = packet.coding_header().first() else {
                return Err(CodingError::DefectPacketError(
                    "Packet should have coding info".into(),
                ));
            };

            self.retrans_queue
                .push_new((info.clone(), packet.data().clone()));
            return Ok(Some(packet));
        }
        Ok(None)
    }
}
