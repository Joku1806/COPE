use std::fmt::Display;
use std::ops::Index;
use std::ops::IndexMut;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct PacketData(Vec<u8>);

impl PacketData {
    pub fn new(raw: Vec<u8>) -> Self {
        Self(raw)
    }

    pub fn right_pad(mut self, len: usize, symbol: u8) -> Self {
        let Some(len_to_pad) = len.checked_sub(self.len()) else {
            return self;
        };
        self.0.extend(vec![symbol; len_to_pad]);
        self
    }

    pub fn xor(mut self, rhs: &PacketData) -> Self {
        for i in 0..usize::min(rhs.0.len(), self.0.len()) {
            self[i] = self[i] ^ rhs[i];
        }
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
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

impl Display for PacketData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // NOTE: Set maximum length to not spam the terminal output
        const MAXLEN: usize = 80;
        // NOTE: non-graphic ASCII characters are replaced by a dot
        // (as it is commonly done in hex editors)
        let human_readable: String = self
            .0
            .iter()
            .take(MAXLEN)
            .map(|b| {
                let ch = *b as char;
                if ch.is_ascii_graphic() {
                    ch
                } else {
                    '.'
                }
            })
            .collect();

        if self.0.len() <= MAXLEN {
            write!(f, "{}", human_readable)
        } else {
            write!(f, "{}<snip>", human_readable)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_xor_same_len() {
        let input0 = PacketData::new(vec![0xFF; 4]);
        let input1 = PacketData::new(vec![0x00; 4]);

        let res0 = input0.clone().xor(&input0);
        let exp0 = PacketData::new(vec![0x00; 4]);
        let res1 = input0.clone().xor(&input1);
        let exp1 = PacketData::new(vec![0xFF; 4]);

        assert_eq!(exp0.0, res0.0);
        assert_eq!(exp1.0, res1.0);
    }

    #[test]
    fn test_xor_different_len() {
        let input0 = PacketData::new(vec![0xFF; 8]);
        let input1 = PacketData::new(vec![0xFF; 4]);
        let input2 = PacketData::new(vec![0x00; 4]);

        let res0 = input0.clone().xor(&input1);
        let mut vec0 = vec![0x00; 4];
        vec0.extend(vec![0xFF; 4]);
        let exp0 = PacketData::new(vec0);
        let res1 = input0.clone().xor(&input2);
        let exp1 = PacketData::new(vec![0xFF; 8]);

        assert_eq!(exp0.0, res0.0);
        assert_eq!(exp1.0, res1.0);
    }

    #[test]
    fn test_right_pad() {
        let input = PacketData::new(vec![0xFF, 0xFF, 0xFF]);
        let res0 = input.clone().right_pad(1, 0);
        let res1 = input.clone().right_pad(3, 0);
        let res2 = input.clone().right_pad(8, 0);
        let exp0 = PacketData::new(vec![0xFF, 0xFF, 0xFF]);
        let exp1 = PacketData::new(vec![0xFF, 0xFF, 0xFF]);
        let exp2 = PacketData::new(vec![0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0]);
        assert_eq!(exp0.0, res0.0);
        assert_eq!(exp1.0, res1.0);
        assert_eq!(exp2.0, res2.0);
    }
}
