use rand::prelude::*;
use std::{cmp::min, ops::Deref};

const FIRST_FRAME_HEADER_SIZE: usize = 7;
const FOLLOWING_FRAME_HEADER_SIZE: usize = 6;

#[derive(Debug)]
pub enum FrameCollectionError {
    FrameMissing,
    FrameAlreadyAdded,
    InvalidFrameSize,
    InvalidFrameCount(usize),
}

// NOTE: Maybe it would be better if each frame contained
// the index and length, not just the first frame.
#[derive(PartialEq, Clone)]
pub enum FrameType {
    First((u8, u8)),
    Following,
}

#[derive(Debug)]
pub enum FrameError {
    InvalidHeader,
}

#[derive(PartialEq, Clone)]
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
        if bytes.len() < FOLLOWING_FRAME_HEADER_SIZE {
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

        let data_start = match ftype {
            FrameType::First(_) => FIRST_FRAME_HEADER_SIZE,
            FrameType::Following => FOLLOWING_FRAME_HEADER_SIZE,
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
            FrameType::First(_) => vec![0; FIRST_FRAME_HEADER_SIZE],
            FrameType::Following => vec![0; FOLLOWING_FRAME_HEADER_SIZE],
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
    frame_size: Option<usize>,
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
        frame_size: usize,
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

        if self.frames.get(index).is_some() {
            return Err(FrameCollectionError::FrameAlreadyAdded);
        }

        match frame.ftype {
            FrameType::First((frame_count, _)) => {
                self.frames.resize_with(frame_count as usize, || None);
            }
            FrameType::Following => (),
        };

        self.frames[index] = Some(frame);

        Ok(())
    }

    pub fn write_first_frame_info(&mut self) -> Result<(), FrameCollectionError> {
        if !self.is_complete() {
            return Err(FrameCollectionError::FrameMissing);
        }

        let frame_count: u8 = match self.frames.len().try_into() {
            Ok(fc) => fc,
            Err(_) => return Err(FrameCollectionError::InvalidFrameCount(self.frames.len())),
        };

        // TODO: Better error handling
        let frame_size = match self.frame_size {
            Some(fs) => match fs.try_into() {
                Ok(fs) => fs,
                Err(_) => return Err(FrameCollectionError::InvalidFrameCount(self.frames.len())),
            },
            None => return Err(FrameCollectionError::InvalidFrameSize),
        };

        let first_frame = match self.frames[0].as_mut() {
            Some(f) => f,
            None => return Err(FrameCollectionError::InvalidFrameCount(self.frames.len())),
        };

        first_frame.ftype = FrameType::First((frame_count, frame_size));

        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        return !self.frames.is_empty() && !self.frames.contains(&None);
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

        while start < bytes.len() {
            let ftype = match index {
                // NOTE: While creating the FrameCollection, we don't exactly know
                // how many frames it will contain. So 0 is just a placeholder
                // and will be changed in write_first_frame_info().
                0 => FrameType::First((0, 0)),
                _ => FrameType::Following,
            };

            let header_size = match ftype {
                FrameType::First(_) => FIRST_FRAME_HEADER_SIZE,
                FrameType::Following => FOLLOWING_FRAME_HEADER_SIZE,
            };

            // NOTE: To be sure that the entire encoded frame will be no more than frame_size bytes long,
            // we need to subtract the header size beforehand.
            let target_size = match self.frame_size {
                Some(fs) => fs - header_size,
                None => return Err(FrameCollectionError::InvalidFrameSize),
            };
            let size = min(target_size, bytes.len() - start);
            let frame = Frame::new(ftype, index, magic, Vec::from(&bytes[start..start + size]));
            self.add_frame(frame)?;

            if start + size > bytes.len() {
                self.write_first_frame_info()?;
            }

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
