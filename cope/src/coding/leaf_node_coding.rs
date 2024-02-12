use std::time::Instant;

use crate::{
    coding::decode_util::{decode, remove_from_pool},
    config::CONFIG,
    packet::{packet::CodingHeader, Ack, CodingInfo, PacketBuilder, PacketData},
    packet_pool::{PacketPool, SimplePacketPool},
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
    last_packet_send: Instant,
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
            last_packet_send: Instant::now(),
        }
    }

    fn should_tx_control(&self) -> bool {
        if self.acks.len() == 0 {
            return false;
        }
        self.last_packet_send.elapsed() > CONFIG.control_packet_duration
    }
}

impl CodingStrategy for LeafNodeCoding {
    fn handle_rx(
        &mut self,
        packet: &Packet,
        topology: &Topology,
    ) -> Result<Option<PacketData>, CodingError> {
        let original_data = packet.data().clone();
        let is_from_relay = packet.sender() == topology.relay();
        if !is_from_relay {
            // store for coding
            return Ok(Some(original_data));
        }
        // handle acks
        let acks = packet.ack_header();
        for ack in acks {
            for info in ack.packets() {
                log::debug!("[Node {}]: Packet {} was acked.", topology.id(), info);
                self.retrans_queue.remove_packet(info);
            }
        }

        log::debug!(
            "[Node{}]: Retrans Queue Size {}",
            topology.id(),
            self.retrans_queue.len()
        );

        match packet.coding_header() {
            CodingHeader::Native(coding_info) => {
                let is_next_hop = topology.id() == coding_info.nexthop;
                if !is_next_hop {
                    // store for coding
                    return Ok(Some(original_data));
                }
            }
            CodingHeader::Encoded(coding_info) => {
                // check if node is next_hop for packet
                if !is_next_hop(topology.id(), coding_info) {
                    log::debug!("[Node {}]: Not a next hop of Packet.", topology.id());
                    return Ok(Some(original_data));
                }
                // decode
                // TODO: add acks to the thing
                let (ids, info) = ids_for_decoding(topology.id(), coding_info, &self.packet_pool)?;
                let decoded_data = decode(&ids, packet.data(), &self.packet_pool);
                log::debug!("[Node {}]: Decoded into {}", topology.id(), decoded_data);
                remove_from_pool(&mut self.packet_pool, &ids);
                self.acks.push(info);
                return Ok(Some(decoded_data));
            }
            CodingHeader::Control(_) => {
                return Ok(Some(original_data));
            }
        }

        Ok(Some(original_data))
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

            let CodingHeader::Native(_info) = packet.coding_header() else {
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

        if self.should_tx_control() {
            let receiver = *topology.txlist().first().unwrap();
            let ack = Ack {
                source: topology.id(),
                packets: std::mem::take(&mut self.acks),
            };
            let result = PacketBuilder::new()
                .sender(topology.id())
                .control_header(receiver)
                .ack_header(vec![ack])
                .build();
            log::debug!("[Relay {}]: Send Control Packet", topology.id());
            match result {
                Ok(control_packet) => return Ok(Some(control_packet)),
                Err(e) => {
                    return Err(CodingError::DefectPacketError(format!(
                        "[Relay {}]: Failed to build Control Packet, because {}",
                        topology.id(),
                        e
                    )))
                }
            }
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

    fn update_last_packet_send(&mut self) {
        self.last_packet_send = Instant::now();
    }
}
