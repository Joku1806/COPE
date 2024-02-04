use std::sync::{Mutex, Arc};

use cope_config::types::node_id::NodeID;

use crate::{
    kbase::{KBase, SimpleKBase},
    packet::{packet::CodingHeader, Ack, CodingInfo, PacketBuilder, PacketData},
    packet_pool::{PacketPool, SimplePacketPool},
    topology::Topology,
    Packet, stats::Stats,
};

use super::{
    retrans_queue::RetransQueue, CodingError, CodingStrategy, QUEUE_SIZE, RETRANS_DURATION,
};

pub struct RelayNodeCoding {
    packet_pool: SimplePacketPool,
    kbase: SimpleKBase,
    retrans_queue: RetransQueue,
    acks: Vec<Ack>,
}

impl RelayNodeCoding {
    pub fn new(tx_list: Vec<NodeID>) -> Self {
        Self {
            packet_pool: SimplePacketPool::new(QUEUE_SIZE),
            kbase: SimpleKBase::new(tx_list, QUEUE_SIZE),
            retrans_queue: RetransQueue::new(QUEUE_SIZE, RETRANS_DURATION),
            acks: vec![],
        }
    }

    fn all_nexhops_can_decode(
        &self,
        packets: &Vec<(CodingInfo, PacketData)>,
        packet: &(CodingInfo, PacketData),
    ) -> bool {
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

    fn has_coding_opp(&self) -> bool {
        // TODO: wait for coding Opportunities
        true
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
        stats: &Arc<Mutex<Stats>>,
    ) -> Result<(), CodingError> {
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
        Ok(())
    }

    fn handle_tx(
        &mut self,
        topology: &Topology,
        stats: &Arc<Mutex<Stats>>,
    ) -> Result<Option<Packet>, CodingError> {
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
            return Ok(None);
        }

        let Some(packet) = self.packet_pool.pop_front() else {
            return Ok(None);
        };
        let coded_packet = self.code_packet(packet, topology)?;

        Ok(Some(coded_packet))
    }
}
