use crate::coding::leaf_node_coding::LeafNodeCoding;
use crate::coding::relay_node_coding::RelayNodeCoding;
use crate::coding::CodingStrategy;
use crate::config::CONFIG;
use crate::stats::Stats;
use crate::topology::Topology;
use crate::traffic_generator::TrafficGenerator;
use crate::{benchmark::BenchTimer, channel::Channel};
use cope_config::types::node_id::NodeID;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Node {
    id: NodeID,
    topology: Topology,
    channel: Box<dyn Channel + Send>,
    coding: Box<dyn CodingStrategy + Send>,
    stats: Arc<Mutex<Stats>>,
    bench: BenchTimer,
}

impl Node {
    pub fn new(
        id: NodeID,
        // NOTE: Send is required for sharing between threads in simulator
        channel: Box<dyn Channel + Send>,
        stats: &Arc<Mutex<Stats>>,
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

        let topology = Topology::new(id, CONFIG.relay, rx_whitelist, tx_whitelist.clone());
        let generator = TrafficGenerator::from_tg_type(tgt, tx_whitelist.clone(), id);
        let coding: Box<dyn CodingStrategy + Send> = match topology.is_relay() {
            true => Box::new(RelayNodeCoding::new(tx_whitelist.clone())),
            false => Box::new(LeafNodeCoding::new(generator)),
        };

        Node {
            id,
            topology,
            channel,
            coding,
            stats: Arc::clone(stats),
            bench: BenchTimer::new(),
        }
    }

    pub fn set_bench_log_path(&mut self, path: &String) {
        self.bench.bench_log_path(path);
    }

    pub fn tick(&mut self) {
        self.receive();
        self.transmit();
        self.bench.log(self.id);
    }

    fn transmit(&mut self) {
        self.bench.record("Transmit handle_tx");
        let packet_to_send = match self.coding.handle_tx(&self.topology) {
            Ok(opt) => opt,
            Err(e) => {
                log::error!("{}", e);
                return;
            }
        };
        self.bench.stop("Transmit handle_tx");
        self.bench.record("Transmit Channel");

        if let Some(packet) = packet_to_send {
            log::debug!("[Node {}]: Send {}", self.id, packet);
            if let Err(e) = self.channel.transmit(&packet) {
                log::error!("{:?}", e);
            } else {
                self.stats.lock().unwrap().add_sent(&packet);
                self.stats.lock().unwrap().log_data();
            }
            self.coding.update_last_packet_send();
            //TODO: handle error
        }
        self.bench.stop("Transmit Channel")
    }

    fn receive(&mut self) {
        // receive
        self.bench.record("Receive Channel");
        if let Some(packet) = self.channel.receive() {
            self.bench.stop("Receive Channel");
            if !self.topology.can_receive_from(packet.sender()) {
                return;
            }

            log::info!("[Node {}]: Received {}", self.id, packet);
            self.bench.record("Receive handle_rx");

            match self.coding.handle_rx(&packet, &self.topology) {
                Ok(Some(data)) => {
                    log::info!("[Node {}]: Decoded data {}", self.id, data);
                    self.stats.lock().unwrap().add_received(
                        packet.sender(),
                        packet.coding_header(),
                        data.len() as u32,
                        true,
                    );
                    self.stats.lock().unwrap().log_data();
                }
                Err(e) => {
                    log::error!("{}", e);
                    self.stats.lock().unwrap().add_received(
                        packet.sender(),
                        packet.coding_header(),
                        packet.data().len() as u32,
                        false,
                    );
                    self.stats.lock().unwrap().log_data();
                }
                _ => (),
            };
            self.bench.stop("Receive handle_rx");
        }
    }
}
