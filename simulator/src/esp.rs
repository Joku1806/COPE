use std::sync::mpsc::{Receiver, Sender};
#[derive(Debug, Clone)]
pub struct Packet {
    pub data: Vec<u8>,
}
pub struct ChannelPacket {
    packet: Packet,
    origin: Mac,
}
impl Packet {
    pub fn new(data: Vec<u8>) -> Self {
        Packet { data: data }
    }
    // really dirty have to do better
    pub fn from_string(data: &str) -> Self {
        Packet {
            data: data.as_bytes().to_vec(),
        }
    }
}
type Mac = [u8; 6];
pub struct ESP {
    pub Rx: Receiver<ChannelPacket>,
    pub Tx: Sender<ChannelPacket>,
    pub mac: Mac,
}
impl ESP {
    pub fn new(rx: Receiver<ChannelPacket>, tx: Sender<ChannelPacket>, mac: Mac) -> Self {
        ESP {
            Rx: rx,
            Tx: tx,
            mac,
        }
    }
    pub fn send(&self, data: Vec<u8>) {
        self.Tx
            .send(ChannelPacket {
                packet: Packet { data: data },
                origin: self.mac,
            })
            .unwrap();
    }
    pub fn recv(&self) -> Option<Vec<u8>> {
        match self.Rx.try_recv() {
            Ok(packet) => Some(packet.packet.data),
            Err(_) => None,
        }
    }
}

