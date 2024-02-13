use cope::channel::Channel;
use cope::packet::Packet;
use std::{
    error::Error,
    sync::mpsc::{Receiver, Sender},
};

pub struct SimulatorChannel {
    rx: Receiver<Packet>,
    tx: Sender<Packet>,
}

// TODO: Figure out if this is needed
unsafe impl Send for SimulatorChannel {}
unsafe impl Sync for SimulatorChannel {}

impl SimulatorChannel {
    pub fn new(rx: Receiver<Packet>, tx: Sender<Packet>) -> Self {
        SimulatorChannel { rx, tx }
    }
}

impl Channel for SimulatorChannel {
    fn transmit(&mut self, packet: &Packet) -> Result<(), Box<dyn Error>> {
        // FIXME: Figure out how to send without cloning
        if let Err(e) = self.tx.send(packet.clone()) {
            println!("{}", e);
        }

        Ok(())
    }

    fn receive(&mut self) -> Option<Packet> {
        self.rx.try_recv().ok()
    }
}
