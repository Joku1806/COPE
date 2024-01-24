use cope::channel::{Channel, ChannelError};
use cope::packet::Packet;
use std::sync::mpsc::{Receiver, Sender};

pub struct SimulatorChannel {
    rx: Receiver<Packet>,
    tx: Sender<Packet>,
}

unsafe impl Send for SimulatorChannel {}
unsafe impl Sync for SimulatorChannel {}

impl SimulatorChannel {
    pub fn new(rx: Receiver<Packet>, tx: Sender<Packet>) -> Self {
        SimulatorChannel { rx, tx }
    }
}

impl Channel for SimulatorChannel {
    fn transmit(&self, packet: &Packet) -> Result<(), ChannelError> {
        // FIXME: Figure out how to send without cloning
        if let Err(e) = self.tx.send(packet.clone()) {
            println!("{}", e);
        }

        Ok(())
    }

    fn receive(&mut self) -> Option<Packet> {
        match self.rx.try_recv() {
            Ok(packet) => Some(packet),
            Err(_) => None,
        }
    }
}
