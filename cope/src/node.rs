
use cope_config::types::node_id::NodeID;
use cope_config::types::traffic_generator_type::TrafficGeneratorType;

use crate::coding::leaf_node_coding::LeafNodeCoding;
use crate::coding::relay_node_coding::RelayNodeCoding;
use crate::coding::CodingStrategy;
use crate::config::CONFIG;
use crate::topology::Topology;
use crate::traffic_generator::greedy_strategy::GreedyStrategy;
use crate::traffic_generator::none_strategy::NoneStrategy;
use crate::traffic_generator::periodic_strategy::PeriodicStrategy;
use crate::traffic_generator::poisson_strategy::PoissonStrategy;
use crate::traffic_generator::random_strategy::RandomStrategy;
use crate::traffic_generator::{TGStrategy, TrafficGenerator};
use crate::channel::Channel;


pub struct Node {
    id: NodeID,
    topology: Topology,
    channel: Box<dyn Channel + Send>,
    coding: Box<dyn CodingStrategy + Send>,
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
        let topology = Topology::new(id, CONFIG.relay, rx_whitelist, tx_whitelist.clone());
        let generator = TrafficGenerator::new(strategy, tx_whitelist.clone(), id);
        let coding: Box<dyn CodingStrategy + Send> = match topology.is_relay() {
            true => Box::new(RelayNodeCoding::new(tx_whitelist.clone())),
            false => Box::new(LeafNodeCoding::new(generator)),
        };

        Node {
            id,
            topology,
            channel,
            coding,
        }
    }

    pub fn tick(&mut self) {
        // send
        let packet_to_send = match self.coding.handle_send(&self.topology) {
            Ok(opt) => opt,
            Err(e) => {
                log::error!("{}", e);
                return;
            }
        };

        if let Some(packet) = packet_to_send {
            log::info!("[Node {}]: Send {:?}", self.id, packet.coding_header());
            self.channel.transmit(&packet);
            //TODO: handle error
        }

        // receive
        if let Some(packet) = self.channel.receive() {
            if !self.topology.can_receive_from(packet.sender()) {
                return;
            }
            log::info!("[Node {}]: Received {:?}", self.id, &packet.coding_header());
            let result = self.coding.handle_receive(&packet, &self.topology);
            if let Err(e) = result {
                log::error!("{}", e);
            }
            //TODO: handle error
        }

    }
}
