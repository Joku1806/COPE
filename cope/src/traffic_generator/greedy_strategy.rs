use crate::packet::PacketBuilder;
use crate::traffic_generator::TGStrategy;

use super::data_generator::DataGenerator;
use super::size_distribution::SizeDistribution;

pub struct GreedyStrategy {
    // NOTE: Distribution of packet sizes
    size_distribution: SizeDistribution,
    data_generator: DataGenerator,
}

// NOTE: A generator that will always return a packet.
// Useful for measuring maximum network throughput.
impl GreedyStrategy {
    pub fn new() -> Self {
        GreedyStrategy {
            size_distribution: SizeDistribution::new(),
            data_generator: DataGenerator::new(),
        }
    }
}

impl TGStrategy for GreedyStrategy {
    fn generate(&mut self) -> Option<PacketBuilder> {
        let target_size = self.size_distribution.sample(&mut rand::thread_rng());
        Some(PacketBuilder::new().data_raw(self.data_generator.generate(target_size)))
    }
}
