use cope_config::types::{node_id::NodeID, traffic_generator_type::TrafficGeneratorType};

use super::packet::PacketBuilder;
use crate::packet::{CodingInfo, PacketID};

pub mod data_generator;
pub mod greedy_strategy;
pub mod none_strategy;
pub mod pareto_strategy;
pub mod periodic_strategy;
pub mod poisson_strategy;
pub mod random_strategy;
pub mod size_distribution;

use greedy_strategy::GreedyStrategy;
use none_strategy::NoneStrategy;
use periodic_strategy::PeriodicStrategy;
use poisson_strategy::PoissonStrategy;
use random_strategy::RandomStrategy;

pub trait TGStrategy {
    fn generate(&mut self) -> Option<PacketBuilder>;
}

// NOTE: This currently uses a round robin aproach to select next tx
// NOTE: But we could also determin next tx using the TGStrategy
pub struct TrafficGenerator {
    strategy: Box<dyn TGStrategy + Send>,
    tx_whitelist: Vec<NodeID>,
    current_tx_id: usize,
    sender_id: NodeID,
    current_packet_id: PacketID,
}

impl TrafficGenerator {
    pub fn new(
        strategy: Box<dyn TGStrategy + Send>,
        tx_whitelist: Vec<NodeID>,
        sender_id: NodeID,
    ) -> Self {
        assert_ne!(tx_whitelist.len(), 0, "tx_whitelist.len() should not be 0");
        TrafficGenerator {
            strategy,
            tx_whitelist,
            current_tx_id: 0,
            sender_id,
            current_packet_id: 0,
        }
    }

    pub fn from_tg_type(
        tgt: TrafficGeneratorType,
        tx_whitelist: Vec<NodeID>,
        sender_id: NodeID,
    ) -> Self {
        let strategy: Box<dyn TGStrategy + Send> = match tgt {
            TrafficGeneratorType::None => Box::new(NoneStrategy::new()),
            TrafficGeneratorType::Greedy => Box::new(GreedyStrategy::new()),
            TrafficGeneratorType::Poisson(rate) => Box::new(PoissonStrategy::new(rate)),
            TrafficGeneratorType::Random(rate) => Box::new(RandomStrategy::new(rate)),
            TrafficGeneratorType::Periodic(duration) => Box::new(PeriodicStrategy::new(duration)),
        };
        TrafficGenerator::new(strategy, tx_whitelist.clone(), sender_id)
    }

    pub fn next_receiver(&mut self) -> NodeID {
        self.current_tx_id = if self.current_tx_id + 1 < self.tx_whitelist.len() {
            self.current_tx_id + 1
        } else {
            0
        };
        self.tx_whitelist[self.current_tx_id]
    }

    pub fn next_packet_id(&mut self) -> PacketID {
        self.current_packet_id = self.current_packet_id.checked_add(1).unwrap_or(0);
        self.current_packet_id
    }

    pub fn generate(&mut self) -> Option<PacketBuilder> {
        self.strategy.generate().map(|builder| {
            builder.sender(self.sender_id).native_header(CodingInfo {
                source: self.sender_id,
                id: self.next_packet_id(),
                nexthop: self.next_receiver(),
            })
        })
    }
}
