use cope_config::types::node_id::NodeID;

use crate::packet::PacketID;

use self::timed_generator::TimedGenerator;
use super::packet::PacketBuilder;

pub mod greedy_generator;
pub mod none_generator;
pub mod pareto_generator;
pub mod poisson_generator;
pub mod random_generator;
pub mod timed_generator;

// NOTE: This currently uses a round robin aproach to select next tx
// NOTE: We can also determin this using the TGStrategy
pub struct TrafficGenerator {
    strategy: Box<dyn TGStrategy + Send>,
    current_packet_id: PacketID,
    tx_whitelist: Vec<NodeID>,
    current_tx_id: usize,
}

impl TrafficGenerator {
    pub fn new(tx_whitelist: Vec<NodeID>) -> Self {
        assert_ne!(tx_whitelist.len(), 0, "tx_whitelist.len() should not be 0");
        TrafficGenerator {
            strategy: Box::new(TimedGenerator::new(std::time::Duration::from_secs(1), 32)),
            current_packet_id: 0,
            tx_whitelist,
            current_tx_id: 0,
        }
    }

    pub fn use_strategy(&mut self) {
        unimplemented!();
        // let generator: Box<dyn TrafficGenerator + Send> = match tgt {
        //     TrafficGeneratorType::None => Box::new(NoneGenerator::new()),
        //     TrafficGeneratorType::Greedy => Box::new(GreedyGenerator::new()),
        //     TrafficGeneratorType::Poisson(mean) => Box::new(PoissonGenerator::new(mean)),
        //     TrafficGeneratorType::Random(mean) => Box::new(RandomGenerator::new(mean as f32)),
        // };
    }

    #[inline]
    pub fn next_receiver(&mut self) -> NodeID {
        self.current_tx_id = if self.current_tx_id+1 < self.tx_whitelist.len() {
            self.current_tx_id + 1
        } else {
            0
        };
        self.tx_whitelist[self.current_tx_id]
    }

    #[inline]
    pub fn next_packet_id(&mut self) -> PacketID {
        self.current_packet_id = self.current_packet_id.checked_add(1).unwrap_or(0);
        self.current_packet_id
    }

    pub fn generate(&mut self) -> Option<PacketBuilder> {
        self.strategy.generate().map(|builder| {
            builder
                .id(self.next_packet_id())
                .receiver(self.next_receiver())
        })
    }
}

pub trait TGStrategy {
    fn generate(&mut self) -> Option<PacketBuilder>;
}
