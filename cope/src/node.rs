use std::collections::VecDeque;

use cope_config::types::node_id::NodeID;

use crate::channel::Channel;
use crate::config::CONFIG;
use crate::packet::Packet;
use crate::topology::Topology;
use crate::traffic_generator::TrafficGenerator;

use chrono::prelude::{Local, DateTime};
use colored::Colorize;
use crate::log;

pub struct Node {
    id: NodeID,
    topology: Topology,
    channel: Box<dyn Channel + Send>,
    is_relay: bool,
    generator: TrafficGenerator,
    packet_fifo: VecDeque<Packet>,
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
            packet_fifo: VecDeque::new(),
        }
    }

    pub fn tick(&mut self) {
        match self.is_relay {
            true => self.tick_relay(),
            false => self.tick_leaf_node(),
        };
    }

    fn tick_relay(&mut self) {
        if let Some(packet) = self.channel.receive() {
            // NOTE: Assuming the relay is able to listen to everything
            log!("[Relay {}]: Recieved {}", self.id, packet.to_info());
            self.packet_fifo.push_back(packet);
        }
        // use coding strategy
        // NOTE: this strategy assumes that there is no error
        while let Some(packet) = self.packet_fifo.pop_front() {
            log!("[Relay {}]: Forwards {}",self.id, packet.to_info());
            self.channel.transmit(&packet.set_sender(self.id));
        }
    }

    fn tick_leaf_node(&mut self) {
        // send
        if let Some(builder) = self.generator.generate() {
            // FIXME: handle this error
            let packet = builder.sender(self.id).build().unwrap();
            log!("[Node {}]: Send {}", self.id, packet.to_info());
            self.channel.transmit(&packet);
        }

        //receive
        if let Some(packet) = self.channel.receive() {
            if self.topology.can_receive_from(packet.sender()) {
                log!("[Node {}]: Received {}", self.id, packet.to_info());
                // decode
                if packet.sender() == CONFIG.relay && packet.receiver() == self.id {
                    // NOTE: Assuming Leaf Nodes don't respond in any way
                    log!("[Node {}]: Got a Message and is very happy!", self.id);
                } else if packet.sender() == CONFIG.relay {
                    log!("[Node {}]: Overheard a Message, that is useless!", self.id);
                } else if packet.receiver() == self.id{
                    log!("[Node {}]: Got a Message via another Node which is strange!", self.id);
                } else {
                    log!("[Node {}]: Overheard a Message, to code with.", self.id);
                }
            }
        }
    }
}
