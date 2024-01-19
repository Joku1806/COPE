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

// NOTE: Maybe it would be better if each frame contained the index and length,
// not just the first frame.
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
        if bytes.len() < FOLLOWING_FRAME_HEADER_SIZE.into() {
            return Err(FrameError::InvalidHeader);
        }

        let magic_bytes = match bytes[0..4].try_into() {
            Ok(m) => m,
            Err(_) => return Err(FrameError::InvalidHeader),
        };
        let magic = u32::from_be_bytes(magic_bytes);

        let ftype = match bytes[4] {
            0 => FrameType::First((bytes[5], bytes[6])),
            1 => FrameType::Following,
            _ => return Err(FrameError::InvalidHeader),
        };

        let index = match ftype {
            FrameType::First(_) => 0,
            FrameType::Following => bytes[5],
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

        v[4] = match self.ftype {
            FrameType::First(_) => 0,
            FrameType::Following => 1,
        };

        v[5] = match self.ftype {
            FrameType::First((frame_count, _)) => frame_count,
            // NOTE: Store inside enum member, like above?
            FrameType::Following => self.index,
        };

        if let FrameType::First((_, frame_size)) = self.ftype {
            v[6] = frame_size;
        }

        v.extend(self.data);

        v
    }
}

#[derive(Clone, Debug)]
pub struct FrameCollection {
    // NOTE: Check if we can just store Vec<Frame>. We would need to check insertion behaviour for
    // that.
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

        // FIXME: Do this without unwrap. In general, the many Optional<> fields in this
        // struct are awkward to work with.
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

            // NOTE: To be sure that the entire encoded frame will be no more than
            // frame_size bytes long, we need to subtract the header size beforehand.
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
                0 => FrameType::First((frame_count, frame_size)),
                _ => FrameType::Following,
            };

            let header_size: usize = match ftype {
                FrameType::First(_) => FIRST_FRAME_HEADER_SIZE.into(),
                FrameType::Following => FOLLOWING_FRAME_HEADER_SIZE.into(),
            };

            // NOTE: To be sure that the entire encoded frame will be no more than
            // frame_size bytes long, we need to subtract the header size beforehand.
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

#[cfg(test)]
mod tests {
    use super::{
        Frame, FrameCollection, FrameCollectionError, FrameType, FOLLOWING_FRAME_HEADER_SIZE,
    };
    use anyhow::Error;

    #[test_case]
    fn first_frame_encode_should_succeed() -> Result<(), Error> {
        let first_frame = Frame::new(
            FrameType::First((1, 4)),
            0,
            0x1f1f1f1f,
            Vec::from([0xfe, 0xed, 0xbe, 0xef]),
        );

        let expected = Vec::from([0x1f, 0x1f, 0x1f, 0x1f, 0, 1, 4, 0xfe, 0xed, 0xbe, 0xef]);
        let actual: Vec<u8> = first_frame.into();

        anyhow::ensure!(
            actual == expected,
            "First frame encoding does not match expected value"
        );

        Ok(())
    }

    #[test_case]
    fn following_frame_encode_should_succeed() -> Result<(), Error> {
        let first_frame = Frame::new(
            FrameType::Following,
            1,
            0xabababab,
            Vec::from([0xde, 0xad, 0xc0, 0xde]),
        );

        let expected = Vec::from([0xab, 0xab, 0xab, 0xab, 1, 1, 0xde, 0xad, 0xc0, 0xde]);
        let actual: Vec<u8> = first_frame.into();

        anyhow::ensure!(
            actual == expected,
            "First frame encoding does not match expected value"
        );

        Ok(())
    }

    #[test_case]
    fn first_frame_decode_should_succeed() -> Result<(), Error> {
        let encoded: [u8; 11] = [0x1f, 0x1f, 0x1f, 0x1f, 0, 1, 4, 0xfe, 0xed, 0xbe, 0xef];

        let expected = Frame::new(
            FrameType::First((1, 4)),
            0,
            0x1f1f1f1f,
            Vec::from([0xfe, 0xed, 0xbe, 0xef]),
        );
        let actual = match Frame::try_from(encoded.as_slice()) {
            Ok(f) => f,
            Err(_) => anyhow::bail!("Could not decode frame"),
        };

        anyhow::ensure!(
            actual == expected,
            "Decoded first frame does not match expected value"
        );

        Ok(())
    }

    #[test_case]
    fn following_frame_decode_should_succeed() -> Result<(), Error> {
        let encoded: [u8; 10] = [0xab, 0xab, 0xab, 0xab, 1, 1, 0xde, 0xad, 0xc0, 0xde];

        let expected = Frame::new(
            FrameType::Following,
            1,
            0xabababab,
            Vec::from([0xde, 0xad, 0xc0, 0xde]),
        );

        let actual = match Frame::try_from(encoded.as_slice()) {
            Ok(f) => f,
            Err(_) => anyhow::bail!("Could not decode frame"),
        };

        anyhow::ensure!(
            actual == expected,
            "Decoded following frame does not match expected value"
        );

        Ok(())
    }

    #[test_case]
    fn frame_size_smaller_than_header_should_fail() -> Result<(), Error> {
        anyhow::ensure!(
            matches!(FrameCollection::new().with_frame_size(FOLLOWING_FRAME_HEADER_SIZE - 1), Err(FrameCollectionError::InvalidFrameSize)),
            "Should not be able to create a FrameCollection with a total size smaller than the smallest header"
        );

        Ok(())
    }

    #[test_case]
    fn mismatched_magic_should_fail() -> Result<(), Error> {
        let mut collection = FrameCollection::new();

        // FIXME: In all tests, FrameType::First((2, 6)) is wrong, but does not affect
        // the test outcome. Should be fixed though.
        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::First((2, 6)),
                    0,
                    0x1f1f1f1f,
                    Vec::from([0x01, 0x23, 0x45]),
                ))
                .is_ok(),
            "mismatched_magic_should_fail: Could not add frame"
        );

        anyhow::ensure!(
            matches!(
                collection.add_frame(Frame::new(
                    FrameType::Following,
                    1,
                    0x23232323,
                    Vec::from([0x45, 0x67, 0x89]),
                )),
                Err(FrameCollectionError::MismatchedMagic(0x23232323))
            ),
            "mismatched_magic_should_fail: Did not return MismatchedMagic Error"
        );

        Ok(())
    }

    #[test_case]
    fn duplicate_index_should_fail() -> Result<(), Error> {
        let mut collection = FrameCollection::new();

        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::First((2, 6)),
                    0,
                    0x1f1f1f1f,
                    Vec::from([0x01, 0x23, 0x45]),
                ))
                .is_ok(),
            "duplicate_index_should_fail: Could not add frame"
        );

        anyhow::ensure!(
            matches!(collection
                .add_frame(Frame::new(
                    FrameType::First((2, 6)),
                    0,
                    0x1f1f1f1f,
                    Vec::from([0x01, 0x23, 0x45]),
                )),
                Err(FrameCollectionError::FrameAlreadyAdded)),
            "duplicate_index_should_fail: Could add duplicate frame, even though it is already present in the collection"
        );

        Ok(())
    }

    #[test_case]
    fn out_of_range_index_should_fail() -> Result<(), Error> {
        let mut collection = FrameCollection::new();

        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::First((2, 6)),
                    0,
                    0x1f1f1f1f,
                    Vec::from([0x01, 0x23, 0x45]),
                ))
                .is_ok(),
            "out_of_range_index_should_fail: Could not add frame"
        );

        anyhow::ensure!(
            matches!(collection
                .add_frame(Frame::new(
                    FrameType::Following,
                    2,
                    0x1f1f1f1f,
                    Vec::from([0x01, 0x23, 0x45]),
                )), Err(FrameCollectionError::InvalidFrameCount(3))),
            "out_of_range_index_should_fail: Could add frame with index 2, even though the collection size is set to 2"
        );

        Ok(())
    }

    #[test_case]
    fn decode_with_missing_index_should_fail() -> Result<(), Error> {
        let mut collection = FrameCollection::new();

        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::First((2, 6)),
                    0,
                    0x13371337,
                    Vec::from([0x01, 0x23, 0x45]),
                ))
                .is_ok(),
            "decode_with_missing_index_should_fail: Could not add frame"
        );

        anyhow::ensure!(
            matches!(collection.decode(), Err(FrameCollectionError::FrameMissing)),
            "decode_with_missing_index_should_fail: Decoding succeded, even though the final frame is missing"
        );

        Ok(())
    }

    #[test_case]
    fn valid_encode_should_succeed() -> Result<(), Error> {
        let data = Vec::from([
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ]);
        let mut collection = match FrameCollection::new()
            .with_magic(0xfeedbeef)
            .with_frame_size(10)
        {
            Ok(fc) => fc,
            Err(_) => {
                anyhow::bail!("valid_encode_should_succeed: Could not construct FrameCollection")
            }
        };

        let expected = Vec::from([
            Frame::new(
                FrameType::First((5, 10)),
                0,
                0xfeedbeef,
                Vec::from([0x00, 0x11, 0x22]),
            ),
            Frame::new(
                FrameType::Following,
                1,
                0xfeedbeef,
                Vec::from([0x33, 0x44, 0x55, 0x66]),
            ),
            Frame::new(
                FrameType::Following,
                2,
                0xfeedbeef,
                Vec::from([0x77, 0x88, 0x99, 0xaa]),
            ),
            Frame::new(
                FrameType::Following,
                3,
                0xfeedbeef,
                Vec::from([0xbb, 0xcc, 0xdd, 0xee]),
            ),
            Frame::new(FrameType::Following, 4, 0xfeedbeef, Vec::from([0xff])),
        ]);

        let enc_result = collection.encode(data.as_slice());
        anyhow::ensure!(
            enc_result.is_ok(),
            "valid_encode_should_succeed: Could not encode data, {:?}",
            enc_result
        );

        let comparable = match collection
            .frames
            .into_iter()
            .collect::<Option<Vec<Frame>>>()
        {
            Some(c) => c,
            None => anyhow::bail!("valid_encode_should_succeed: Could not collect frames"),
        };
        anyhow::ensure!(
            comparable == expected,
            "valid_encode_should_succeed: Bytes are not encoded to expected frame collection"
        );

        Ok(())
    }

    #[test_case]
    fn valid_decode_should_succeed() -> Result<(), Error> {
        let mut collection = FrameCollection::new();

        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::First((2, 6)),
                    0,
                    0x13371337,
                    Vec::from([0x01, 0x23, 0x45]),
                ))
                .is_ok(),
            "valid_decode_should_succeed: Could not add frame"
        );

        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::Following,
                    1,
                    0x13371337,
                    Vec::from([0x45, 0x67, 0x89]),
                ))
                .is_ok(),
            "valid_decode_should_succeed: Could not add frame"
        );

        let expected = Vec::from([0x01, 0x23, 0x45, 0x45, 0x67, 0x89]);
        let decoded = match collection.decode() {
            Ok(d) => d,
            Err(_) => {
                anyhow::bail!("valid_decode_should_succeed: Could not decode FrameCollection")
            }
        };

        anyhow::ensure!(
            decoded == expected,
            "valid_decode_should_succeed: Mismatch when decoding frames"
        );

        Ok(())
    }

    #[test_case]
    fn valid_decode_with_out_of_order_insertion_should_succeed() -> Result<(), Error> {
        let mut collection = FrameCollection::new();

        // TODO: Test for: When following frame is added first, so the frame count is
        // not known yet.
        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::First((3, 9)),
                    0,
                    0x13371337,
                    Vec::from([0x01, 0x23, 0x45]),
                ))
                .is_ok(),
            "valid_decode_with_out_of_order_insertion_should_succeed: Could not add frame"
        );

        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::Following,
                    2,
                    0x13371337,
                    Vec::from([0xab, 0xcd, 0xef]),
                ))
                .is_ok(),
            "valid_decode_with_out_of_order_insertion_should_succeed: Could not add frame"
        );

        anyhow::ensure!(
            collection
                .add_frame(Frame::new(
                    FrameType::Following,
                    1,
                    0x13371337,
                    Vec::from([0x45, 0x67, 0x89]),
                ))
                .is_ok(),
            "valid_decode_with_out_of_order_insertion_should_succeed: Could not add frame"
        );

        let expected = Vec::from([0x01, 0x23, 0x45, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
        let decoded = match collection.decode() {
            Ok(d) => d,
            Err(_) => anyhow::bail!("valid_decode_with_out_of_order_insertion_should_succeed: Could not decode FrameCollection"),
        };

        anyhow::ensure!(decoded == expected, "valid_decode_with_out_of_order_insertion_should_succeed: Mismatch when decoding frames");

        Ok(())
    }
}
