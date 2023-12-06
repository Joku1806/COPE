use cope_config::types::node_id::NodeID;

use crate::packet::{PacketID, CodingInfo};

use super::packet::PacketBuilder;

pub mod greedy_strategy;
pub mod none_strategy;
pub mod pareto_strategy;
pub mod periodic_strategy;
pub mod poisson_strategy;
pub mod random_strategy;

// NOTE: This currently uses a round robin aproach to select next tx
// NOTE: We can also determin this using the TGStrategy
pub struct TrafficGenerator {
    strategy: Box<dyn TGStrategy + Send>,
    tx_whitelist: Vec<NodeID>,
    current_tx_id: usize,
    sender_id: NodeID,
}

impl TrafficGenerator {
    pub fn new(
        strategy: Box<dyn TGStrategy + Send>,
        tx_whitelist: Vec<NodeID>,
        sender_id: NodeID
    ) -> Self {
        assert_ne!(tx_whitelist.len(), 0, "tx_whitelist.len() should not be 0");
        TrafficGenerator {
            strategy,
            tx_whitelist,
            current_tx_id: 0,
            sender_id
        }
    }

    #[inline]
    pub fn next_receiver(&mut self) -> NodeID {
        self.current_tx_id = if self.current_tx_id + 1 < self.tx_whitelist.len() {
            self.current_tx_id + 1
        } else {
            0
        };
        self.tx_whitelist[self.current_tx_id]
    }


    pub fn generate(&mut self) -> Option<PacketBuilder> {
        self.strategy.generate().map(|builder| {
            builder
                .sender(self.sender_id)
                .single_coding_header(self.sender_id, self.next_receiver())
        })
    }
}

pub trait TGStrategy {
    fn generate(&mut self) -> Option<PacketBuilder>;
}
