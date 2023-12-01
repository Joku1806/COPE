use std::vec::Vec;

use crate::channel::Channel;
use crate::topology::{NodeID, Topology};
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
        relay: NodeID,
        allowlist: Vec<NodeID>,
        // NOTE: Send is required for sharing between threads in simulator
        channel: Box<dyn Channel + Send>,
        traffic_generator: Box<dyn TrafficGenerator + Send>,
    ) -> Self {
        Node {
            id,
            topology: Topology::new(id, relay, allowlist),
            channel,
            generator: traffic_generator,
        }
    }

    pub fn tick(&mut self) {
        if let Some(packet) = self.channel.receive() {
            println!("Node {}: Received packet {:?}", self.id, packet);

            if self.topology.can_receive_from('A') {
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
