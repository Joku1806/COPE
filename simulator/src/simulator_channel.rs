use cope::channel::Channel;
use cope::packet::Packet;
use std::sync::mpsc::{Receiver, Sender};

pub struct SimulatorChannel {
    rx: Receiver<Packet>,
    tx: Sender<Packet>,
}

impl SimulatorChannel {
    pub fn new(rx: Receiver<Packet>, tx: Sender<Packet>) -> Self {
        SimulatorChannel { rx, tx }
    }
}

// TODO: Does this need to be implemented?
unsafe impl Sync for SimulatorChannel {}

impl Channel for SimulatorChannel {
    fn transmit(&self, packet: &Packet) {
        // FIXME: Figure out how to send without cloning
        self.tx.send(packet.clone()).unwrap();
    }

    fn receive(&self) -> Option<Packet> {
        match self.rx.try_recv() {
            Ok(packet) => Some(packet),
            Err(_) => None,
        }
    }
}
