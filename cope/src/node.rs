use cope_config::types::node_id::NodeID;
// use cope_config::types::traffic_generator_type::TrafficGeneratorType;

use crate::channel::Channel;
use crate::config::CONFIG;
use crate::packet::Packet;
use crate::topology::Topology;
// use crate::traffic_generator::greedy_generator::GreedyGenerator;
// use crate::traffic_generator::none_generator::NoneGenerator;
// use crate::traffic_generator::poisson_generator::PoissonGenerator;
// use crate::traffic_generator::random_generator::RandomGenerator;
use crate::traffic_generator::timed_generator::TimedGenerator;
use crate::traffic_generator::TrafficGenerator;

pub struct Node {
    id: NodeID,
    topology: Topology,
    channel: Box<dyn Channel + Send>,
    is_relay: bool,
    generator: TrafficGenerator,
    packet_fifo: Vec<Packet>,
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
        eprintln!("{:?}:{:?}",&id, &tx_whitelist);

        let _tgt = CONFIG
            .get_generator_type_for(id)
            .expect("Config should contain traffic generator type");

        // TODO: add an is_relay() -> bool method to config struct
        let is_relay = CONFIG.relay == id;

        let generator = TrafficGenerator::new(tx_whitelist);

        Node {
            id,
            topology: Topology::new(id, CONFIG.relay, rx_whitelist),
            channel,
            is_relay,
            generator,
            packet_fifo: Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        match self.is_relay {
            true => self.tick_relay(),
            false => self.tick_leaf_node(),
        };

        // if let Some(packet) = self.channel.receive() {
        //     if self.topology.can_receive_from(packet.get_sender()) {
        //         println!("Node {}: Received packet {:?}", self.id, packet);
        //         // TODO: network coding
        //     }
        // }

        //     packet.set_sender(self.id);
        //     println!("Node {}: Sending packet {:?}", self.id, packet);
        //     self.channel.transmit(&packet);
        // }
    }

    fn tick_relay(&mut self) {
        if let Some(packet) = self.channel.receive() {
            println!("[Relay {}]: recieved package {:?}", self.id, packet);
        }
    }

    fn tick_leaf_node(&mut self) {
        if let Some(builder) = self.generator.generate() {
            // FIXME: handle this error
            let packet = builder.sender(self.id).build().unwrap();
            self.channel.transmit(&packet);
            println!("[Node {}]: wants to send package", self.id);
        }
    }
}
