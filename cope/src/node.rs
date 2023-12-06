use std::collections::{VecDeque, HashMap};
use std::usize;

use cope_config::types::node_id::NodeID;
use cope_config::types::traffic_generator_type::TrafficGeneratorType;
use rand_distr::Poisson;

use crate::{channel::Channel, packet::CodingInfo};
use crate::config::CONFIG;
use crate::packet::{Packet, PacketBuilder};
use crate::topology::Topology;
use crate::traffic_generator::none_strategy::NoneStrategy;
use crate::traffic_generator::periodic_strategy::PeriodicStrategy;
use crate::traffic_generator::poisson_strategy::PoissonStrategy;
use crate::traffic_generator::random_strategy::RandomStrategy;
use crate::traffic_generator::{TGStrategy, TrafficGenerator};
use crate::{traffic_generator::greedy_strategy::GreedyStrategy};
use super::packet_pool::{PacketPool, SimplePacketPool};

use crate::log;
use chrono::prelude::{DateTime, Local};
use colored::Colorize;

const MAX_PACKET_POOL_SIZE: usize = 8;

struct KnowledgeBase {
    table: HashMap<NodeID, Vec<CodingInfo>>,
    max_size: usize,
}

impl KnowledgeBase {
    fn new(next_hops: Vec<NodeID>, max_size: usize) -> Self{
        let table = next_hops.iter().map(|&i| (i, vec![])).collect();
        Self { table, max_size }
    }

    fn knows(&self, next_hop: &NodeID, info: &CodingInfo) -> bool{
        self.table.get(next_hop)
            .expect("knowledge_base should have a filed for every node!")
            .contains(info)
    }

    fn insert(&mut self, next_hop: NodeID, info: CodingInfo){
        let list = self.table.get_mut(&next_hop)
            .expect("KnowledgeBase should contain Entry for nexthop");
        let is_at_max_size = list.len() >= self.max_size;
        if  is_at_max_size { list.remove(0); }
        list.push(info);
    }

    fn size(&self) -> usize{
        self.table.iter().map(|(_, list)| list.len()).sum()
    }
}


pub struct Node {
    id: NodeID,
    topology: Topology,
    channel: Box<dyn Channel + Send>,
    is_relay: bool,
    generator: TrafficGenerator,
    tx_whitelist: Vec<NodeID>,
    packet_pool: SimplePacketPool,
    last_fifo_flush: std::time::Instant,
    knowledge_base: KnowledgeBase
}

fn xor_data(mut a: Vec<u8>, b: &Vec<u8>) -> Vec<u8> {
    for i in 0..usize::min(b.len(), a.len()) {
        a[i] = a[i] ^ b[i];
    }
    a
}

impl Node {
    pub fn new(
        id: NodeID,
        // NOTE: Send is required for sharing between threads in simulator
        channel: Box<dyn Channel + Send>,
    ) -> Self {
        let rx_whitelist = CONFIG
            .get_rx_whitelist_for(id)
            .expect("Config should contain rx whitelist");

        // TODO: Pass into TrafficGenerator, so it can randomly choose receivers
        // NOTE: I did this, but currently we just use round robin
        let tx_whitelist = CONFIG
            .get_tx_whitelist_for(id)
            .expect("Config should contain tx whitelist");
        eprintln!("{:?}:{:?}", &id, &tx_whitelist);

        let tgt = CONFIG
            .get_generator_type_for(id)
            .expect("Config should contain traffic generator type");

        let strategy: Box<dyn TGStrategy + Send> = match tgt {
            TrafficGeneratorType::None => Box::new(NoneStrategy::new()),
            TrafficGeneratorType::Greedy => Box::new(GreedyStrategy::new()),
            TrafficGeneratorType::Poisson(rate) => Box::new(PoissonStrategy::new(rate)),
            TrafficGeneratorType::Random(rate) => Box::new(RandomStrategy::new(rate)),
            TrafficGeneratorType::Periodic(duration) => Box::new(PeriodicStrategy::new(duration)),
        };
        // TODO: add an is_relay() -> bool method to config struct
        let is_relay = CONFIG.relay == id;

        let generator = TrafficGenerator::new(strategy, tx_whitelist.clone(), id);

        Node {
            id,
            topology: Topology::new(id, CONFIG.relay, rx_whitelist),
            channel,
            is_relay,
            generator,
            tx_whitelist: tx_whitelist.clone(),
            last_fifo_flush: std::time::Instant::now(),
            packet_pool: SimplePacketPool::new(MAX_PACKET_POOL_SIZE),
            knowledge_base: KnowledgeBase::new(tx_whitelist.clone(), MAX_PACKET_POOL_SIZE),
        }
    }

    pub fn tick(&mut self) {
        match self.is_relay {
            true => self.tick_relay(),
            false => self.tick_leaf_node(),
        };
    }


    fn tick_relay(&mut self) {
        // receive
        if let Some(packet) = self.channel.receive() {
            log!("[Relay {}]: Recieved {:?}", self.id, packet.coding_header());
            // TODO: extract acks ment for retransmission event
            // TODO: extract reception reports
            let coding_info = packet.coding_header().first().unwrap();
            let data = packet.data();
            // append knowledge base
            self.knowledge_base.insert(packet.sender(), coding_info.clone());
            // add to packet pool
            self.packet_pool.push_packet(packet);
            log!("[Relay {}]: Has stored {} packages and knows about {}", self.id, self.packet_pool.size(), self.knowledge_base.size());
        }

        // NOTE: we always take some time before we flush fifo to enable coding opportunities
        let  time_elapsed = self.last_fifo_flush.elapsed();
        if time_elapsed < std::time::Duration::from_millis(800) { return; }
        self.last_fifo_flush = std::time::Instant::now();
        log!("[Relay {}]: Attepts to forward packets.", self.id);

        // deque head of output queue
        if let Some(packet) = self.packet_pool.pop_front() {
            log!("[Relay {}]: Starts forwarding packet {:?}", self.id, packet.0);
            log!("[Relay {}]: Looking for Coding Opportunities", self.id);

            let mut packets: Vec<(CodingInfo, Vec<u8>)> = vec![packet];
            for &nexthop in &self.tx_whitelist {
                let Some(packet_i) = self.packet_pool.peek_nexthop_front(nexthop)
                else { continue; };

                if self.all_nexhops_can_decode(&packets, packet_i) {
                    let p = self.packet_pool.pop_nexthop_front(nexthop).unwrap();
                    packets.push(p);
                }
            }
            log!("[Relay {}]: Found {} packets to code", self.id, packets.len());
            // Remove packets from packet_pool
            // Encode if possible
            let (header, data) = self.encode(&packets);
            log!("[Relay {}]: Encoded to {:?}, {:?}", self.id ,&header, &data);
            // TODO: add acks to header
            let pack = PacketBuilder::new()
                .sender(self.id)
                .data(data)
                .coding_header(header)
                .build()
                .unwrap();
            // If encoded schedule retransmission
            // send
            self.channel.transmit(&pack);
            log!("[Relay {}]: Forwarded package ", self.id);
        }else {
            log!("[Relay {}]: No Packets to forward", self.id);
        }
    }

    fn encode(&self, packets: &Vec<(CodingInfo, Vec<u8>)>) -> (Vec<CodingInfo>, Vec<u8>) {
        let info = packets
            .iter()
            .cloned()
            .map(|p| p.0)
            .collect();
        let data = packets
            .iter()
            .cloned()
            .map(|p| p.1)
            .fold(packets[0].1.clone(), |acc, x| xor_data(acc, &x));
        (info, data)
    }

    // NOTE: Assume that for each next_hop we only add 1 Package
    // therefore return true if each next_hop knows exactly packets.add(packet).len()-1
    fn all_nexhops_can_decode( &self,
        packets: &Vec<(CodingInfo, Vec<u8>)>,
        packet: &(CodingInfo, Vec<u8>),
    ) -> bool {
        let iter = std::iter::once(packet).chain(packets);
        for (CodingInfo{ nexthop, ..},_) in iter {
            let iter1 = std::iter::once(packet).chain(packets);
            for (info, _) in iter1 {
                let knows = self.knowledge_base.knows(nexthop, info);
                let is_nexthop = *nexthop == info.nexthop;
                if !knows && ! is_nexthop { return false; }
            }
        }
        true
    }


    fn tick_leaf_node(&mut self) {
        // send
        if let Some(builder) = self.generator.generate() {
            // FIXME: handle this error
            let packet = builder.build().unwrap();
            log!("[Node {}]: Send {:?}", self.id, &packet.coding_header());
            // TODO: add reception report
            self.channel.transmit(&packet);
            let coding_info = packet.coding_header().first().unwrap();
            self.packet_pool.push_packet(packet);
            log!("[Node {}]: Has stored {} packages.", self.id, self.packet_pool.size());
        }

        // receive
        if let Some(packet) = self.channel.receive() {
            if self.topology.can_receive_from(packet.sender()) {
                log!("[Node {}]: Received {:?}", self.id, packet.coding_header());
                if packet.sender() == CONFIG.relay{
                    // decode
                    if !self.is_next_hop(&packet) {
                        log!("[Node {}]: Not a next hop of Packet.", self.id);
                    } else if let Some(data) = self.decode(&packet) {
                        log!("[Node {}]: Decoded Packet to {:?}.", self.id, data);
                    } else {
                        log!("[Node {}]: Could not decode Packet.", self.id);
                    }
                    log!("[Node {}]: Has stored {} packages.", self.id, self.packet_pool.size());
                } else {
                    //store for coding
                }
            }
        }
    }

    fn is_next_hop(&self, packet: &Packet) -> bool {
        packet.coding_header().iter().find(|&x| x.nexthop == self.id).is_some()
    }

    // FIXME: Refactor this mess of a function
    fn decode(&mut self, packet: &Packet) -> Option<Vec<u8>> {
        let mut packet_indices: Vec<usize> = vec![];
        for info in packet.coding_header() {
            let Some(index) = self.packet_pool.position(&info)
            else{
                if info.nexthop == self.id { continue; }
                return None;
            };
            packet_indices.push(index);
        }
        if packet_indices.len() != packet.coding_header().len() - 1 {
            return None
        }

        let mut data: Vec<u8> = packet.data().clone();

        for &index in &packet_indices {
            let (_, d) = self.packet_pool.remove(index).unwrap();
            data = xor_data(data, &d);
        }
        return Some(data);
    }
}
