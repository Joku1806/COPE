use std::time::Duration;

use cope_config::types::node_id::NodeID;

use crate::{
    kbase::{KBase, SimpleKBase},
    packet::{CodingInfo, PacketBuilder, PacketData},
    packet_pool::{PacketPool, SimplePacketPool},
    topology::Topology,
    Packet,
};

use super::{retrans_queue::RetransQueue, CodingError, CodingStrategy};

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
            retrans_queue: RetransQueue::new(8, Duration::from_millis(1000)),
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

    // NOTE: id is just for logging
    fn packet_to_send(&mut self, id: NodeID) -> Option<(CodingInfo, PacketData)> {
        if let Some((info, data)) = self.retrans_queue.packet_to_retrans() {
            log::info!(
                "[Relay {}], Packet {:?} was not acked. Retransimitting",
                id,
                info
            );
            return Some((info, data));
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
                log::info!("[Relay {}]: Packet {:?} was acked.", topology.id(), info);
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

        let Some(packet_to_send) = self.packet_to_send(topology.id()) else {
            return Ok(None);
        };

        log::info!(
            "[Relay {}]: Starts forwarding packet {:?}",
            topology.id(),
            packet_to_send.0
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
            "[Relay {}]: Found {} packets to code.",
            topology.id(),
            packets.len()
        );
        // Remove packets from packet_pool
        // Encode if possible
        let (header, data) = encode(&packets);
        log::info!("[Relay {}]: Encoded to {:?}.", topology.id(), &header,);
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
