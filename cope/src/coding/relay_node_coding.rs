use std::time;

use cope_config::types::node_id::NodeID;

use crate::{
    kbase::{KBase, SimpleKBase},
    packet::{CodingInfo, PacketBuilder, PacketData},
    packet_pool::{PacketPool, SimplePacketPool},
    topology::Topology,
    Packet,
};

use super::{CodingError, CodingStrategy};

pub struct RetransEntry {
    data: PacketData,
    info: CodingInfo,
    retrans_count: u8,
    last_trans: time::Instant,
}

pub struct RetransQueue {
    queue: Vec<RetransEntry>,
}

impl RetransQueue {
    fn new() -> Self {
        Self { queue: vec![] }
    }

    fn is_not_full(&self) -> bool {
        unimplemented!();
    }

    fn packet_to_retrans(&self) -> Option<&(CodingInfo, PacketData)> {
        unimplemented!();
    }

    fn push_new(&self, packet: (CodingInfo, PacketData)) {
        unimplemented!();
    }

    fn remove_packet(&self, info: &CodingInfo) {
        todo!()
    }

}

pub struct RelayNodeCoding {
    packet_pool: SimplePacketPool,
    kbase: SimpleKBase,
    retrans_queue: RetransQueue,
}

impl RelayNodeCoding {
    pub fn new(tx_list: Vec<NodeID>) -> Self {
        Self {
            packet_pool: SimplePacketPool::new(8),
            kbase: SimpleKBase::new(tx_list, 8),
            retrans_queue: RetransQueue::new(),
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

    fn packet_to_send(&mut self) -> Option<(CodingInfo, PacketData)> {
        if let Some(next_packet) = self.retrans_queue.packet_to_retrans() {
            return Some(next_packet.clone());
        }
        if let Some(next_packet) = self.packet_pool.pop_front() {
            return Some(next_packet);
        }
        None
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
    fn handle_receive(&mut self, packet: &Packet, topology: &Topology) -> Result<(), CodingError> {
        let coding_info = packet.coding_header().first().unwrap();
        // TODO: handle Acks
        let acks = packet.ack_header();
        for ack in acks {
            for info in ack.packets() {
                self.retrans_queue.remove_packet(info);
            }
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

    fn handle_send(&mut self, topology: &Topology) -> Result<Option<Packet>, CodingError> {
        // TODO: wait for coding Opportunities

        let Some(packet_to_send) = self.packet_to_send() else {
            return Ok(None);
        };

        log::info!(
            "[Relay {}]: Starts forwarding packet {:?}",
            topology.id(),
            packet_to_send.0
        );
        log::info!(
            "[Relay {}]: Looking for Coding Opportunities",
            topology.id()
        );

        let mut packets: Vec<(CodingInfo, PacketData)> = vec![packet_to_send];
        for &nexthop in topology.txlist() {
            let Some(packet) = self.packet_pool.peek_nexthop_front(nexthop) else {
                continue;
            };

            if self.all_nexhops_can_decode(&packets, packet) {
                let p = self.packet_pool.pop_nexthop_front(nexthop).unwrap();
                packets.push(p);
            }
        }
        log::info!(
            "[Relay {}]: Found {} packets to code",
            topology.id(),
            packets.len()
        );
        // Remove packets from packet_pool
        // Encode if possible
        let (header, data) = encode(&packets);
        log::info!(
            "[Relay {}]: Encoded to {:?}, {:?}",
            topology.id(),
            &header,
            &data
        );
        // TODO: add acks to header

        // schedule retransmission
        for p in &packets {
            self.retrans_queue.push_new(p.clone());
        }

        let coded_packet = PacketBuilder::new()
            .sender(topology.id())
            .data(data)
            .coding_header(header)
            .build()
            .unwrap();

        Ok(Some(coded_packet))
    }
}
