use std::ops::IndexMut;
use std::ops::Index;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct PacketData(Vec<u8>);

impl PacketData {
    // creational
    pub fn new(raw: Vec<u8>) -> Self {
        Self(raw)
    }

    // mutation
    pub fn padding(mut self, len: usize) -> Self{
        todo!();
    }

    pub fn xor(mut self, rhs: &PacketData) -> Self{
        for i in 0..usize::min(rhs.0.len(), self.0.len()) {
            self[i] = self[i] ^ rhs[i];
        }
        self
    }

    //
    pub fn size(&self) -> usize { self.0.len() }
}

impl Index<usize> for PacketData {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for PacketData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
