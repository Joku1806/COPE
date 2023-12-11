#[derive(Debug)]
pub enum EspressifWifiFrameDecodingError {
    InvalidLength,
    InvalidSigMode,
    InvalidChannelBandwidth,
    InvalidChannelEstimateSmootingValue,
    InvalidPPDUType,
    InvalidAggregationType,
    InvalidSTBCValue,
    InvalidGuideInterval,
}

#[derive(Default)]
enum SigMode {
    #[default]
    HT11bg,
    HT11n,
    VHT11ac,
}

impl TryFrom<u32> for SigMode {
    type Error = EspressifWifiFrameDecodingError;

    fn try_from(sig_mode: u32) -> Result<Self, Self::Error> {
        match sig_mode {
            0 => Ok(Self::HT11bg),
            1 => Ok(Self::HT11n),
            2 => Ok(Self::VHT11ac),
            _ => Err(Self::Error::InvalidSigMode),
        }
    }
}

#[derive(Default)]
enum ChannelBandwidth {
    #[default]
    MHz20,
    MHz40,
}

impl TryFrom<u32> for ChannelBandwidth {
    type Error = EspressifWifiFrameDecodingError;

    fn try_from(bw: u32) -> Result<Self, Self::Error> {
        match bw {
            0 => Ok(Self::MHz20),
            1 => Ok(Self::MHz40),
            _ => Err(Self::Error::InvalidChannelBandwidth),
        }
    }
}

#[derive(Default)]
enum ChannelEstimateSmoothing {
    #[default]
    Recommended,
    NotRecommended,
}

impl TryFrom<u32> for ChannelEstimateSmoothing {
    type Error = EspressifWifiFrameDecodingError;

    fn try_from(rec: u32) -> Result<Self, Self::Error> {
        match rec {
            0 => Ok(Self::NotRecommended),
            1 => Ok(Self::Recommended),
            _ => Err(Self::Error::InvalidChannelEstimateSmootingValue),
        }
    }
}

#[derive(Default)]
enum PPDUType {
    #[default]
    Sounding,
    NotSounding,
}

impl TryFrom<u32> for PPDUType {
    type Error = EspressifWifiFrameDecodingError;

    fn try_from(ppdu_type: u32) -> Result<Self, Self::Error> {
        match ppdu_type {
            0 => Ok(Self::Sounding),
            1 => Ok(Self::NotSounding),
            _ => Err(Self::Error::InvalidPPDUType),
        }
    }
}

#[derive(Default)]
enum AggregationType {
    #[default]
    MPDU,
    AMPDU,
}

impl TryFrom<u32> for AggregationType {
    type Error = EspressifWifiFrameDecodingError;

    fn try_from(aggregation_type: u32) -> Result<Self, Self::Error> {
        match aggregation_type {
            0 => Ok(Self::MPDU),
            1 => Ok(Self::AMPDU),
            _ => Err(Self::Error::InvalidAggregationType),
        }
    }
}

#[derive(Default)]
enum STBC {
    #[default]
    Yes,
    No,
}

impl TryFrom<u32> for STBC {
    type Error = EspressifWifiFrameDecodingError;

    fn try_from(stbc: u32) -> Result<Self, Self::Error> {
        match stbc {
            0 => Ok(Self::No),
            1 => Ok(Self::Yes),
            _ => Err(Self::Error::InvalidSTBCValue),
        }
    }
}

#[derive(Default)]
enum GuideInterval {
    #[default]
    Short,
    Long,
}

impl TryFrom<u32> for GuideInterval {
    type Error = EspressifWifiFrameDecodingError;

    fn try_from(interval: u32) -> Result<Self, Self::Error> {
        match interval {
            0 => Ok(Self::Long),
            1 => Ok(Self::Short),
            _ => Err(Self::Error::InvalidGuideInterval),
        }
    }
}

#[derive(Default)]
struct RadioMetadataHeader {
    rssi: i32,
    rate: u32,
    __pad0__: u32,
    sig_mode: SigMode,
    __pad1__: u32,
    mcs: u32,
    cwb: ChannelBandwidth,
    __pad2__: u32,
    smoothing: ChannelEstimateSmoothing,
    ppdu_type: PPDUType,
    __pad3__: u32,
    aggregation: AggregationType,
    stbc: STBC,
    fec_coding: u32,
    gi: GuideInterval,
    noise_floor: i32,
    ampdu_cnt: u32,
    channel: u32,
    secondary_channel: u32,
    __pad4__: u32,
    // NOTE: If this is micros since device startup, converting to DateTime is meaningless
    timestamp_us: u32,
    __pad5__: u32,
    __pad6__: u32,
    ant: u32,
    sig_len: u32,
    __pad7__: u32,
    rx_state: u32,
}

// FIXME: This is the common header at the beginning of all promiscuous mode RX callback buffers, which is only specific to Espressif.
// It is not part of an IEEE 802.11 Frame, this struct/file should be renamed to something else!
#[derive(Default)]
pub struct EspressifWifiFrame {
    header: RadioMetadataHeader,
    data: Vec<u8>,
}

impl TryFrom<&[u8]> for EspressifWifiFrame {
    type Error = EspressifWifiFrameDecodingError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        const HEADER_LENGTH: usize = 108;

        if bytes.len() < HEADER_LENGTH {
            return Err(EspressifWifiFrameDecodingError::InvalidLength);
        }

        let sig_len = u32::from_be_bytes(bytes[92..96].try_into().unwrap());

        // FIXME: Need to check for wrapping errors
        if bytes.len() != HEADER_LENGTH + sig_len as usize {
            return Err(EspressifWifiFrameDecodingError::InvalidLength);
        }

        let mut frame = EspressifWifiFrame::default();

        // TODO: Return all errors by coercing to a common error type somehow, instead of panicking
        frame.header.rssi = i32::from_be_bytes(bytes[0..4].try_into().unwrap());
        frame.header.rate = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        frame.header.sig_mode = u32::from_be_bytes(bytes[12..16].try_into().unwrap()).try_into()?;
        frame.header.mcs = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        frame.header.cwb = u32::from_be_bytes(bytes[24..28].try_into().unwrap()).try_into()?;
        frame.header.smoothing =
            u32::from_be_bytes(bytes[32..36].try_into().unwrap()).try_into()?;
        frame.header.ppdu_type =
            u32::from_be_bytes(bytes[36..40].try_into().unwrap()).try_into()?;
        frame.header.aggregation =
            u32::from_be_bytes(bytes[44..48].try_into().unwrap()).try_into()?;
        frame.header.stbc = u32::from_be_bytes(bytes[48..52].try_into().unwrap()).try_into()?;
        frame.header.fec_coding = u32::from_be_bytes(bytes[52..56].try_into().unwrap());
        frame.header.gi = u32::from_be_bytes(bytes[56..60].try_into().unwrap()).try_into()?;
        frame.header.noise_floor = i32::from_be_bytes(bytes[56..60].try_into().unwrap());
        frame.header.ampdu_cnt = u32::from_be_bytes(bytes[60..64].try_into().unwrap());
        frame.header.channel = u32::from_be_bytes(bytes[64..68].try_into().unwrap());
        frame.header.secondary_channel = u32::from_be_bytes(bytes[68..72].try_into().unwrap());
        frame.header.timestamp_us = u32::from_be_bytes(bytes[76..80].try_into().unwrap());
        frame.header.ant = u32::from_be_bytes(bytes[88..92].try_into().unwrap());
        frame.header.sig_len = sig_len;
        frame.header.rx_state = u32::from_be_bytes(bytes[100..104].try_into().unwrap());

        frame.data = Vec::from(&bytes[104..]);

        Ok(frame)
    }
}
