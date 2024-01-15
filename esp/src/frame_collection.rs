use rand::prelude::*;
use std::{cmp::min, ops::Deref};

const FIRST_FRAME_HEADER_SIZE: u8 = 7;
const FOLLOWING_FRAME_HEADER_SIZE: u8 = 6;

#[derive(Debug)]
pub enum FrameCollectionError {
    FrameMissing,
    FrameAlreadyAdded,
    InvalidFrameSize,
    InvalidFrameCount(usize),
    UnsetMagic,
    MismatchedMagic(u32),
}

// NOTE: Maybe it would be better if each frame contained
// the index and length, not just the first frame.
#[derive(PartialEq, Clone, Debug)]
pub enum FrameType {
    First((u8, u8)),
    Following,
}

#[derive(Debug)]
pub enum FrameError {
    InvalidHeader,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Frame {
    ftype: FrameType,
    index: u8,
    magic: u32,
    data: Vec<u8>,
}

impl Frame {
    pub fn new(ftype: FrameType, index: u8, magic: u32, data: Vec<u8>) -> Frame {
        Frame {
            ftype,
            index,
            magic,
            data,
        }
    }

    pub fn get_magic(&self) -> u32 {
        self.magic
    }
}

impl TryFrom<&[u8]> for Frame {
    type Error = FrameError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        // FIXME: This check does not work anymore, now that there are two different header sizes. We first need to determine
        if bytes.len() < FOLLOWING_FRAME_HEADER_SIZE.into() {
            return Err(FrameError::InvalidHeader);
        }

        let magic_bytes = match bytes[0..4].try_into() {
            Ok(m) => m,
            Err(_) => return Err(FrameError::InvalidHeader),
        };
        let magic = u32::from_be_bytes(magic_bytes);

        let ftype = match bytes[5] {
            0 => FrameType::First((bytes[6], bytes[7])),
            1 => FrameType::Following,
            _ => return Err(FrameError::InvalidHeader),
        };

        let index = match ftype {
            FrameType::First(_) => 0,
            FrameType::Following => bytes[6],
        };

        let data_start: usize = match ftype {
            FrameType::First(_) => FIRST_FRAME_HEADER_SIZE.into(),
            FrameType::Following => FOLLOWING_FRAME_HEADER_SIZE.into(),
        };

        Ok(Frame {
            ftype,
            index,
            magic,
            data: bytes[data_start..].to_vec(),
        })
    }
}

impl Into<Vec<u8>> for Frame {
    fn into(self) -> Vec<u8> {
        let mut v = match self.ftype {
            FrameType::First(_) => vec![0; FIRST_FRAME_HEADER_SIZE.into()],
            FrameType::Following => vec![0; FOLLOWING_FRAME_HEADER_SIZE.into()],
        };

        v[0..4].clone_from_slice(&u32::to_be_bytes(self.magic));

        v[5] = match self.ftype {
            FrameType::First(_) => 0,
            FrameType::Following => 1,
        };

        v[6] = match self.ftype {
            FrameType::First((frame_count, _)) => frame_count,
            // NOTE: Store inside enum member, like above?
            FrameType::Following => self.index,
        };

        if let FrameType::First((_, frame_size)) = self.ftype {
            v.push(frame_size);
        }

        v.extend(self.data);

        v
    }
}

#[derive(Clone)]
pub struct FrameCollection {
    // NOTE: Check if we can just store Vec<Frame>.
    // We would need to check insertion behaviour for that.
    frames: Vec<Option<Frame>>,
    frame_size: Option<u8>,
    magic: Option<u32>,
}

impl FrameCollection {
    pub fn new() -> FrameCollection {
        FrameCollection {
            frames: Vec::new(),
            frame_size: None,
            magic: None,
        }
    }

    pub fn with_frame_size(
        mut self,
        frame_size: u8,
    ) -> Result<FrameCollection, FrameCollectionError> {
        if frame_size < FOLLOWING_FRAME_HEADER_SIZE {
            return Err(FrameCollectionError::InvalidFrameSize);
        }

        self.frame_size = Some(frame_size);
        Ok(self)
    }

    pub fn with_magic(mut self, magic: u32) -> FrameCollection {
        self.magic = Some(magic);
        self
    }

    pub fn add_frame(&mut self, frame: Frame) -> Result<(), FrameCollectionError> {
        let index = frame.index as usize;

        if frame.ftype == FrameType::Following {
            if index >= self.frames.len() {
                return Err(FrameCollectionError::InvalidFrameCount(index + 1));
            }

            let magic = match self.magic {
                Some(m) => m,
                None => return Err(FrameCollectionError::UnsetMagic),
            };

            if magic != frame.get_magic() {
                return Err(FrameCollectionError::MismatchedMagic(frame.get_magic()));
            }
        }

        // FIXME: Do this without unwrap. In general, the many Optional<> fields in this struct are awkward to work with
        if index < self.frames.len() && self.frames.get(index).unwrap().is_some() {
            return Err(FrameCollectionError::FrameAlreadyAdded);
        }

        match frame.ftype {
            FrameType::First((frame_count, _)) => {
                self.frames.resize_with(frame_count as usize, || None);
                self.magic = Some(frame.get_magic());
            }
            FrameType::Following => (),
        };

        self.frames[index] = Some(frame);

        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        return !self.frames.is_empty() && !self.frames.contains(&None);
    }

    fn calculate_frame_count(&self, data_len: usize) -> Result<u8, FrameCollectionError> {
        let mut frame_count: u8 = 0;
        let mut start = 0;
        let frame_size = match self.frame_size {
            Some(fs) => Into::<usize>::into(fs),
            None => return Err(FrameCollectionError::InvalidFrameSize),
        };

        while start < data_len {
            let ftype = match start {
                0 => FrameType::First((0, 0)),
                _ => FrameType::Following,
            };

            let header_size: usize = match ftype {
                FrameType::First(_) => FIRST_FRAME_HEADER_SIZE.into(),
                FrameType::Following => FOLLOWING_FRAME_HEADER_SIZE.into(),
            };

            // NOTE: To be sure that the entire encoded frame will be no more than frame_size bytes long,
            // we need to subtract the header size beforehand.
            let target_size = frame_size - header_size;
            let size = min(target_size, data_len - start);
            start += size;
            frame_count = match frame_count.checked_add(1) {
                Some(fc) => fc,
                None => {
                    return Err(FrameCollectionError::InvalidFrameCount(
                        Into::<usize>::into(frame_count) + 1,
                    ))
                }
            }
        }

        Ok(frame_count)
    }

    // NOTE: Encodes from flat packet representation to FrameCollection
    pub fn encode(&mut self, bytes: &[u8]) -> Result<(), FrameCollectionError> {
        if !self.frames.is_empty() {
            return Err(FrameCollectionError::FrameAlreadyAdded);
        }

        let mut start: usize = 0;
        let mut index: u8 = 0;
        let magic: u32 = match self.magic {
            None => rand::thread_rng().gen(),
            Some(m) => m,
        };

        let frame_size = match self.frame_size {
            Some(fs) => fs,
            None => return Err(FrameCollectionError::InvalidFrameSize),
        };
        let frame_count = self.calculate_frame_count(bytes.len())?;

        while start < bytes.len() {
            let ftype = match index {
                // NOTE: While creating the FrameCollection, we don't exactly know
                // how many frames it will contain. So 0 is just a placeholder
                // and will be changed in write_first_frame_info().
                0 => FrameType::First((frame_count, frame_size)),
                _ => FrameType::Following,
            };

            let header_size: usize = match ftype {
                FrameType::First(_) => FIRST_FRAME_HEADER_SIZE.into(),
                FrameType::Following => FOLLOWING_FRAME_HEADER_SIZE.into(),
            };

            // NOTE: To be sure that the entire encoded frame will be no more than frame_size bytes long,
            // we need to subtract the header size beforehand.
            let target_size = Into::<usize>::into(frame_size) - header_size;
            let size = min(target_size, bytes.len() - start);
            let frame = Frame::new(ftype, index, magic, Vec::from(&bytes[start..start + size]));
            self.add_frame(frame)?;

            start += size;
            index += 1;
        }

        Ok(())
    }

    // NOTE: Decodes from FrameCollection to flat packet representation
    pub fn decode(&self) -> Result<Vec<u8>, FrameCollectionError> {
        let mut decoded = Vec::new();

        if !self.is_complete() {
            return Err(FrameCollectionError::FrameMissing);
        }

        for frame in self.frames.iter() {
            decoded.extend(frame.clone().unwrap().data);
        }

        Ok(decoded)
    }
}

impl Deref for FrameCollection {
    type Target = Vec<Option<Frame>>;

    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}
