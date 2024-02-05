use std::sync::{Arc, Mutex};

use crate::{
    coding::{
        self,
        decode_util::{decode, remove_from_pool},
    },
    config::CONFIG,
    packet::{packet::CodingHeader, Ack, CodingInfo, PacketBuilder, PacketData},
    packet_pool::{PacketPool, SimplePacketPool},
    stats::Stats,
    topology::Topology,
    traffic_generator::TrafficGenerator,
    Packet,
};

use super::{
    decode_util::{ids_for_decoding, is_next_hop},
    retrans_queue::RetransQueue,
    CodingError, CodingStrategy,
};

pub struct LeafNodeCoding {
    generator: TrafficGenerator,
    packet_pool: SimplePacketPool,
    retrans_queue: RetransQueue,
    acks: Vec<CodingInfo>,
}

impl LeafNodeCoding {
    pub fn new(generator: TrafficGenerator) -> Self {
        let sz = CONFIG.packet_pool_size;
        let rtt = CONFIG.round_trip_time;

        Self {
            generator,
            packet_pool: SimplePacketPool::new(sz),
            retrans_queue: RetransQueue::new(sz, rtt),
            acks: vec![],
        }
    }
}

impl CodingStrategy for LeafNodeCoding {
    fn handle_rx(
        &mut self,
        packet: &Packet,
        topology: &Topology,
    ) -> Result<Option<PacketData>, CodingError> {
        let is_from_relay = packet.sender() == topology.relay();
        if !is_from_relay {
            //store for coding
            return Ok(None);
        }
        // handle acks
        let acks = packet.ack_header();
        for ack in acks {
            for info in ack.packets() {
                log::info!("[Node {}]: Packet {:?} was acked.", topology.id(), info);
                self.retrans_queue.remove_packet(info);
            }
        }
        log::info!(
            "[Node{}]: Retrans Queue Size {:?}",
            topology.id(),
            self.retrans_queue.len()
        );

        match packet.coding_header() {
            CodingHeader::Native(coding_info) => {
                let is_next_hop = topology.id() == coding_info.nexthop;
                if !is_next_hop {
                    // store for coding
                    return Ok(None);
                }
            }
            CodingHeader::Encoded(coding_info) => {
                // check if node is next_hop for packet
                if !is_next_hop(topology.id(), coding_info) {
                    log::info!("[Node {}]: Not a next hop of Packet.", topology.id());
                    return Ok(None);
                }
                // decode
                // TODO: add acks to the thing
                let (ids, info) = ids_for_decoding(topology.id(), coding_info, &self.packet_pool)?;
                let decoded_data = decode(&ids, packet.data(), &self.packet_pool);
                log::info!("[Node {}]: Decoded into {:?}", topology.id(), decoded_data);
                remove_from_pool(&mut self.packet_pool, &ids);
                self.acks.push(info);
                return Ok(Some(decoded_data));
            }
            CodingHeader::Control => {
                return Ok(None);
            }
        }

        Ok(Some(packet.data().clone()))
    }

    fn handle_tx(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError> {
        if let Some((info, data)) = self.retrans_queue.packet_to_retrans() {
            let builder = PacketBuilder::new()
                .sender(topology.id())
                .data(data)
                .native_header(info);
            // TODO: add reception report

            // add acks to header
            let ack = Ack {
                source: topology.id(),
                packets: std::mem::take(&mut self.acks),
            };

            let packet = builder.ack_header(vec![ack]).build().unwrap();
            self.packet_pool.push_packet(packet.clone());

            let CodingHeader::Native(info) = packet.coding_header() else {
                return Err(CodingError::DefectPacketError(
                    "Expected to retransmit Native Packet".into(),
                ));
            };
            return Ok(Some(packet));
        }

        if self.retrans_queue.is_full() {
            return Err(CodingError::FullRetransQueue(format!(
                "[Node {}]: Cannot send new packet, without dropping old Packet.",
                topology.id()
            )));
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

            let CodingHeader::Native(info) = packet.coding_header() else {
                return Err(CodingError::DefectPacketError(
                    "Expected to send Native Packet".into(),
                ));
            };

            self.retrans_queue
                .push_new((info.clone(), packet.data().clone()));
            return Ok(Some(packet));
        }
        Ok(None)
    }
}
