use crate::topology::NodeID;

#[derive(Debug, Clone)]

pub struct Packet {
    pub data: Vec<u8>,
    pub origin: NodeID,
}

impl Packet {
    pub fn new(data: Vec<u8>) -> Self {
        Packet { data, origin: '*' }
    }

    pub fn from_string(data: &str) -> Self {
        Packet {
            data: data.as_bytes().to_vec(),
            origin: '*',
        }
    }

    pub fn set_sender(&mut self, id: NodeID) {
        self.origin = id;
    }
}
