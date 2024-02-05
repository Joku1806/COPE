use std::time::Instant;

use cope_config::types::node_id::NodeID;

use crate::{
    config::CONFIG,
    kbase::{KBase, SimpleKBase},
    packet::{packet::CodingHeader, Ack, CodingInfo, PacketBuilder, PacketData},
    packet_pool::{PacketPool, SimplePacketPool},
    topology::Topology,
    Packet,
};

use super::{retrans_queue::RetransQueue, CodingError, CodingStrategy};

pub struct RelayNodeCoding {
    packet_pool: SimplePacketPool,
    kbase: SimpleKBase,
    retrans_queue: RetransQueue,
    acks: Vec<Ack>,
    last_packet_send: Instant,
}

impl RelayNodeCoding {
    pub fn new(tx_list: Vec<NodeID>) -> Self {
        let sz = CONFIG.packet_pool_size;
        let rtt = CONFIG.round_trip_time;

        Self {
            packet_pool: SimplePacketPool::new(sz),
            kbase: SimpleKBase::new(tx_list, sz),
            retrans_queue: RetransQueue::new(sz, rtt),
            acks: vec![],
            last_packet_send: Instant::now(),
        }
    }

    fn all_nexhops_can_decode(
        &self,
        packets: &Vec<(CodingInfo, PacketData)>,
        packet: &(CodingInfo, PacketData),
    ) -> bool {
        if packets
            .iter()
            .find(|(c, _)| c.nexthop == packet.0.nexthop)
            .is_some()
        {
            return false;
        }

        let iter = std::iter::once(packet).chain(packets);

        for (CodingInfo { nexthop, .. }, _) in iter {
            let iter1 = std::iter::once(packet).chain(packets);
            for (info, _) in iter1 {
                let knows = self.kbase.knows(nexthop, info);
                let is_nexthop = *nexthop == info.nexthop;

                if !knows && !is_nexthop {
                    return false;
                }
            }
        }
        true
    }

    fn should_tx_control(&self) -> bool {
        if self.acks.len() == 0 {
            return false;
        }
        self.last_packet_send.elapsed() > CONFIG.control_packet_duration
    }

    fn has_coding_opp(&self) -> bool {
        // FIXME: This can definitely be improved
        self.packet_pool.size() >= 2
    }

    fn code_packet(
        &mut self,
        packet: (CodingInfo, PacketData),
        topo: &Topology,
    ) -> Result<Packet, CodingError> {
        let mut packets: Vec<(CodingInfo, PacketData)> = vec![packet];
        for &nexthop in topo.txlist() {
            let Some(packet) = self.packet_pool.peek_nexthop_front(nexthop) else {
                continue;
            };

            if self.all_nexhops_can_decode(&packets, packet) {
                let p = self.packet_pool.pop_nexthop_front(nexthop).unwrap();
                packets.push(p);
            }
        }
        let (header, data) = encode(&packets);
        // schedule retransmission
        for p in &packets {
            if self.retrans_queue.conatains(&p.0) {
                self.retrans_queue.push_new(p.clone());
            }
        }

        let coded_packet = PacketBuilder::new()
            .sender(topo.id())
            .data(data)
            .encoded_header(header)
            .ack_header(std::mem::take(&mut self.acks))
            .build()
            .unwrap();
        log::info!("[Relay {}]: {:?}", topo.id(), coded_packet.ack_header());

        Ok(coded_packet)
    }
}

fn encode(packets: &Vec<(CodingInfo, PacketData)>) -> (Vec<CodingInfo>, PacketData) {
    let info = packets.iter().cloned().map(|p| p.0).collect();
    let data = packets
        .iter()
        .cloned()
        .map(|p| p.1)
        .fold(packets[0].1.clone(), |acc, x| acc.xor(&x));
    (info, data)
}

impl CodingStrategy for RelayNodeCoding {
    fn handle_rx(
        &mut self,
        packet: &Packet,
        topology: &Topology,
    ) -> Result<Option<PacketData>, CodingError> {
        if let CodingHeader::Control(_) = packet.coding_header() {
            let acks = packet.ack_header();
            for ack in acks {
                for info in ack.packets() {
                    log::info!("[Relay {}]: Packet {:?} was acked.", topology.id(), info);
                    self.retrans_queue.remove_packet(info);
                }
                self.acks.push(ack.clone());
            }
            return Ok(None);
        }

        let CodingHeader::Native(coding_info) = packet.coding_header() else {
            return Err(CodingError::DefectPacketError(
                "Expected to receive Native Packet".into(),
            ));
        };

        let acks = packet.ack_header();
        for ack in acks {
            for info in ack.packets() {
                log::info!("[Relay {}]: Packet {:?} was acked.", topology.id(), info);
                self.retrans_queue.remove_packet(info);
            }
            self.acks.push(ack.clone());
        }
        // append knowledge base
        self.kbase.insert(packet.sender(), coding_info.clone());
        // add to packet pool
        self.packet_pool.push_packet(packet.clone());
        log::info!(
            "[Relay {}]: Has stored {} packages and knows about {}",
            topology.id(),
            self.packet_pool.size(),
            self.kbase.size()
        );

        Ok(Some(packet.data().clone()))
    }

    fn handle_tx(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError> {
        if let Some(packet) = self.retrans_queue.packet_to_retrans() {
            let coded_packet = self.code_packet(packet, topology)?;
            return Ok(Some(coded_packet));
        }

        if self.retrans_queue.is_full() {
            return Err(CodingError::FullRetransQueue(format!(
                "[Relay {}]:Cannot send new packet, without dropping old Packet.",
                topology.id()
            )));
        }

        if !self.has_coding_opp() {
            if self.should_tx_control() {
                let receiver = *topology.txlist().first().unwrap();
                let result = PacketBuilder::new()
                    .sender(topology.id())
                    .control_header(receiver)
                    .ack_header(std::mem::take(&mut self.acks))
                    .build();
                log::info!("[Relay {}]: Send Control Packet", topology.id());
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
            return Ok(None);
        }

        let Some(packet) = self.packet_pool.pop_front() else {
            return Ok(None);
        };
        let coded_packet = self.code_packet(packet, topology)?;

        Ok(Some(coded_packet))
    }

    fn update_last_packet_send(&mut self) {
        self.last_packet_send = Instant::now();
    }
}
