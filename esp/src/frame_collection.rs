use rand::prelude::*;
use std::{cmp::min, ops::Deref};

// NOTE: Maximum allowed frame size defined by EspNow
const FRAME_MAX_SIZE: usize = 250;
const FRAME_HEADER_SIZE: usize = 6;

#[derive(Debug)]
pub enum FrameCollectionError {
    FrameMissing,
    FrameAlreadyAdded,
}

#[derive(PartialEq, Clone)]
pub enum FrameType {
    First(u8),
    Following,
}

#[derive(Debug)]
pub enum FrameError {
    InvalidHeader,
    InvalidDataLength,
}

#[derive(PartialEq, Clone)]
pub struct Frame {
    ftype: FrameType,
    index: u8,
    magic: u32,
    data: Vec<u8>,
}

impl Frame {
    pub fn new(
        ftype: FrameType,
        index: u8,
        magic: u32,
        data: Vec<u8>,
    ) -> Result<Frame, FrameError> {
        if data.len() + FRAME_HEADER_SIZE > FRAME_MAX_SIZE {
            return Err(FrameError::InvalidDataLength);
        }

        Ok(Frame {
            ftype,
            index,
            magic,
            data,
        })
    }

    pub fn get_magic(&self) -> u32 {
        self.magic
    }
}

impl TryFrom<&[u8]> for Frame {
    type Error = FrameError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < FRAME_HEADER_SIZE {
            return Err(FrameError::InvalidHeader);
        }

        if bytes.len() > FRAME_MAX_SIZE {
            return Err(FrameError::InvalidDataLength);
        }

        let ftype = match bytes[0] {
            0 => FrameType::First(bytes[1]),
            1 => FrameType::Following,
            _ => return Err(FrameError::InvalidHeader),
        };

        let index = match ftype {
            FrameType::First(_) => 0,
            FrameType::Following => bytes[1],
        };

        let magic = u32::from_be_bytes(bytes[2..6].try_into().unwrap());

        Ok(Frame {
            ftype,
            index,
            magic,
            data: bytes[FRAME_HEADER_SIZE..].to_vec(),
        })
    }
}

impl Into<Vec<u8>> for Frame {
    fn into(self) -> Vec<u8> {
        let mut v = vec![0; FRAME_HEADER_SIZE];

        v[0] = match self.ftype {
            FrameType::First(_) => 0,
            FrameType::Following => 1,
        };

        v[1] = match self.ftype {
            FrameType::First(total) => total,
            FrameType::Following => self.index,
        };

        v[2..6].clone_from_slice(&u32::to_be_bytes(self.magic));

        v.extend(self.data);

        v
    }
}

#[derive(Clone)]
pub struct FrameCollection {
    frames: Vec<Option<Frame>>,
}

impl FrameCollection {
    pub fn new() -> FrameCollection {
        FrameCollection { frames: Vec::new() }
    }

    pub fn add_frame(&mut self, frame: Frame) -> Result<(), FrameCollectionError> {
        let index = frame.index as usize;

        if self.frames.get(index).is_some() {
            return Err(FrameCollectionError::FrameAlreadyAdded);
        }

        match frame.ftype {
            FrameType::First(total) => {
                self.frames.resize_with(total as usize, || None);
            }
            FrameType::Following => (),
        };

        self.frames[index] = Some(frame);

        Ok(())
    }

    pub fn write_frame_counter(&mut self) {
        assert!(
            self.is_complete(),
            "Can only be called on completed frame collection",
        );
        assert!(
            self.frames.len() < u8::MAX.into(),
            "We only support single byte frame lengths"
        );

        let length = self.frames.len() as u8;
        let first = self.frames[0].as_mut().unwrap();
        first.ftype = FrameType::First(length);
    }

    pub fn is_complete(&self) -> bool {
        return !self.frames.is_empty() && !self.frames.contains(&None);
    }
}

impl From<&[u8]> for FrameCollection {
    fn from(bytes: &[u8]) -> Self {
        let mut collection = FrameCollection::new();
        let mut start: usize = 0;
        let mut index: u8 = 0;
        let magic: u32 = rand::thread_rng().gen();

        while start < bytes.len() {
            let size = min(FRAME_MAX_SIZE, bytes.len() - start);
            let ftype = match index {
                // NOTE: While creating the FrameCollection, we don't exactly know
                // how many frames it will contain. So 0 is just a placeholder
                // and will be changed in write_frame_counter().
                0 => FrameType::First(0),
                _ => FrameType::Following,
            };
            // TODO: Error handling
            let frame =
                Frame::new(ftype, index, magic, Vec::from(&bytes[start..start + size])).unwrap();
            collection.add_frame(frame).unwrap();

            if start + size > bytes.len() {
                collection.write_frame_counter();
            }

            start += size;
            index += 1;
        }

        collection
    }
}

impl TryInto<Vec<u8>> for FrameCollection {
    type Error = FrameCollectionError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        if !self.is_complete() {
            return Err(Self::Error::FrameMissing);
        }

        let mut encoded = Vec::new();

        for frame in self.frames.iter() {
            // TODO: Get this to work without cloning
            encoded.extend::<Vec<u8>>(frame.clone().unwrap().into());
        }

        Ok(encoded)
    }
}

impl Deref for FrameCollection {
    type Target = Vec<Option<Frame>>;

    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}
