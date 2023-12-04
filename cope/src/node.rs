use cope_config::types::node_id::NodeID;
use cope_config::types::traffic_generator_type::TrafficGeneratorType;

use crate::channel::Channel;
use crate::config::CONFIG;
use crate::topology::Topology;
use crate::traffic_generator::greedy_generator::GreedyGenerator;
use crate::traffic_generator::none_generator::NoneGenerator;
use crate::traffic_generator::poisson_generator::PoissonGenerator;
use crate::traffic_generator::random_generator::RandomGenerator;
use crate::traffic_generator::TrafficGenerator;

pub struct Node {
    id: NodeID,
    topology: Topology,
    channel: Box<dyn Channel + Send>,
    generator: Box<dyn TrafficGenerator + Send>,
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
        let _tx_whitelist = CONFIG
            .get_tx_whitelist_for(id)
            .expect("Config should contain tx whitelist");

        let tgt = CONFIG
            .get_generator_type_for(id)
            .expect("Config should contain traffic generator type");

        let receivers = CONFIG
            .get_tx_whitelist_for(id)
            .expect("Config should contain rx whitelist");

        // TODO: Find a way to move this to TrafficGenerator trait
        let generator: Box<dyn TrafficGenerator + Send> = match tgt {
            TrafficGeneratorType::None => Box::new(NoneGenerator::new()),
            TrafficGeneratorType::Greedy => Box::new(GreedyGenerator::new(receivers)),
            TrafficGeneratorType::Poisson(mean) => Box::new(PoissonGenerator::new(mean, receivers)),
            TrafficGeneratorType::Random(mean) => {
                Box::new(RandomGenerator::new(mean as f32, receivers))
            }
        };

        Node {
            id,
            topology: Topology::new(id, CONFIG.relay, rx_whitelist),
            channel,
            generator,
        }
    }

    pub fn tick(&mut self) {
        if let Some(packet) = self.channel.receive() {
            if self.topology.can_receive_from(packet.get_sender()) {
                println!("Node {}: Received packet {:?}", self.id, packet);
                // TODO: network coding
            }
        }

        if let Some(mut packet) = self.generator.generate() {
            packet.set_sender(self.id);
            println!("Node {}: Sending packet {:?}", self.id, packet);
            self.channel.transmit(&packet);
        }
    }
}
