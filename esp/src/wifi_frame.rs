use bitvec::{field::BitField, prelude as bv, view::BitView};

#[derive(Debug)]
pub enum WifiFrameDecodingError {
    InvalidSigMode,
    InvalidChannelBandwidth,
    InvalidChannelEstimateSmootingValue,
    InvalidPPDUType,
    InvalidAggregationType,
    InvalidSTBCValue,
    InvalidGuideInterval,
}

#[derive(Default, Debug)]
enum SigMode {
    #[default]
    HT11bg,
    HT11n,
    VHT11ac,
}

impl TryFrom<u32> for SigMode {
    type Error = WifiFrameDecodingError;

    fn try_from(sig_mode: u32) -> Result<Self, Self::Error> {
        match sig_mode {
            0 => Ok(Self::HT11bg),
            1 => Ok(Self::HT11n),
            2 => Ok(Self::VHT11ac),
            _ => Err(Self::Error::InvalidSigMode),
        }
    }
}

#[derive(Default, Debug)]
enum ChannelBandwidth {
    #[default]
    MHz20,
    MHz40,
}

impl TryFrom<u32> for ChannelBandwidth {
    type Error = WifiFrameDecodingError;

    fn try_from(bw: u32) -> Result<Self, Self::Error> {
        match bw {
            0 => Ok(Self::MHz20),
            1 => Ok(Self::MHz40),
            _ => Err(Self::Error::InvalidChannelBandwidth),
        }
    }
}

#[derive(Default, Debug)]
enum ChannelEstimateSmoothing {
    #[default]
    Recommended,
    NotRecommended,
}

impl TryFrom<u32> for ChannelEstimateSmoothing {
    type Error = WifiFrameDecodingError;

    fn try_from(rec: u32) -> Result<Self, Self::Error> {
        match rec {
            0 => Ok(Self::NotRecommended),
            1 => Ok(Self::Recommended),
            _ => Err(Self::Error::InvalidChannelEstimateSmootingValue),
        }
    }
}

#[derive(Default, Debug)]
enum PPDUType {
    #[default]
    Sounding,
    NotSounding,
}

impl TryFrom<u32> for PPDUType {
    type Error = WifiFrameDecodingError;

    fn try_from(ppdu_type: u32) -> Result<Self, Self::Error> {
        match ppdu_type {
            0 => Ok(Self::Sounding),
            1 => Ok(Self::NotSounding),
            _ => Err(Self::Error::InvalidPPDUType),
        }
    }
}

#[derive(Default, Debug)]
enum AggregationType {
    #[default]
    MPDU,
    AMPDU,
}

impl TryFrom<u32> for AggregationType {
    type Error = WifiFrameDecodingError;

    fn try_from(aggregation_type: u32) -> Result<Self, Self::Error> {
        match aggregation_type {
            0 => Ok(Self::MPDU),
            1 => Ok(Self::AMPDU),
            _ => Err(Self::Error::InvalidAggregationType),
        }
    }
}

#[derive(Default, Debug)]
enum STBC {
    #[default]
    Yes,
    No,
}

impl TryFrom<u32> for STBC {
    type Error = WifiFrameDecodingError;

    fn try_from(stbc: u32) -> Result<Self, Self::Error> {
        match stbc {
            0 => Ok(Self::No),
            1 => Ok(Self::Yes),
            _ => Err(Self::Error::InvalidSTBCValue),
        }
    }
}

#[derive(Default, Debug)]
enum GuideInterval {
    #[default]
    Short,
    Long,
}

impl TryFrom<u32> for GuideInterval {
    type Error = WifiFrameDecodingError;

    fn try_from(interval: u32) -> Result<Self, Self::Error> {
        match interval {
            0 => Ok(Self::Long),
            1 => Ok(Self::Short),
            _ => Err(Self::Error::InvalidGuideInterval),
        }
    }
}

#[derive(Default, Debug)]
struct RadioMetadataHeader {
    rssi: i32,
    rate: u32,
    sig_mode: SigMode,
    mcs: u32,
    cwb: ChannelBandwidth,
    smoothing: ChannelEstimateSmoothing,
    ppdu_type: PPDUType,
    aggregation: AggregationType,
    stbc: STBC,
    fec_coding: u32,
    gi: GuideInterval,
    noise_floor: i32,
    ampdu_cnt: u32,
    channel: u32,
    secondary_channel: u32,
    // NOTE: If this is micros since device startup, converting to DateTime is meaningless
    timestamp_us: u32,
    ant: u32,
    sig_len: u32,
    rx_state: u32,
}

// FIXME: This is the common header at the beginning of all promiscuous mode RX
// callback buffers, which is only specific to Espressif. It is not part of an
// IEEE 802.11 Frame, this struct/file should be renamed to something else!
#[derive(Default, Debug)]
pub struct WifiFrame {
    header: RadioMetadataHeader,
    data: Vec<u8>,
}

impl TryFrom<&[u8]> for WifiFrame {
    type Error = WifiFrameDecodingError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut frame = WifiFrame::default();
        const HEADER_SIZE: usize = 48;
        let header = &bytes[..HEADER_SIZE];
        let bits = header.view_bits::<bv::Lsb0>();

        // TODO: Return all errors by coercing to a common error type somehow, instead
        // of panicking
        frame.header.rssi = bits[0..8].load::<i32>();
        frame.header.rate = bits[8..13].load::<u32>();
        frame.header.sig_mode = bits[14..16].load::<u32>().try_into()?;
        frame.header.mcs = bits[14..21].load::<u32>();
        frame.header.cwb = bits[39..40].load::<u32>().try_into()?;
        frame.header.smoothing = bits[56..57].load::<u32>().try_into()?;
        frame.header.ppdu_type = bits[57..58].load::<u32>().try_into()?;
        frame.header.aggregation = bits[59..60].load::<u32>().try_into()?;
        frame.header.stbc = bits[60..62].load::<u32>().try_into()?;
        frame.header.fec_coding = bits[62..63].load::<u32>();
        frame.header.gi = bits[63..64].load::<u32>().try_into()?;
        frame.header.noise_floor = bits[160..168].load::<i32>();
        frame.header.ampdu_cnt = bits[72..80].load::<u32>();
        frame.header.channel = bits[80..84].load::<u32>();
        frame.header.secondary_channel = bits[84..88].load::<u32>();
        frame.header.timestamp_us = bits[96..128].load::<u32>();
        frame.header.ant = bits[255..256].load::<u32>();
        frame.header.sig_len = bits[352..364].load::<u32>();
        frame.header.rx_state = bits[376..384].load::<u32>();

        frame.data = Vec::from(&bytes[HEADER_SIZE..]);

        Ok(frame)
    }
}

impl WifiFrame {
    pub fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }
}
