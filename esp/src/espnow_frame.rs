use cope_config::types::mac_address::MacAddress;

#[derive(Default, Debug)]
struct EspNowMacHeader {
    frame_control: u16,
    duration_or_id: u16,
    destination: MacAddress,
    source: MacAddress,
    broadcast: MacAddress,
    sequence_control: u16,
}

#[derive(Default, Debug)]
struct EspNowVendorContent {
    element_id: u8,
    length: u8,
    organization_identifier: [u8; 3],
    vc_type: u8,
    version: u8,
    body: Vec<u8>,
}

#[derive(Default, Debug)]
pub struct EspNowFrame {
    mac_header: EspNowMacHeader,
    category_code: u8,
    organization_identifier: [u8; 3],
    random: u32,
    vendor_content: EspNowVendorContent,
    fcs: u32,
}

#[derive(Debug)]
pub enum EspNowDecodingError {
    InvalidLength,
    InvalidFrameControl,
    InvalidBroadcastMAC,
    InvalidCategoryCode,
    InvalidOrganizationIdentifier,
    InvalidElementId,
    InvalidVendorContentType,
}

pub const ESPNOW_HEADER_SIZE: usize = 39;

impl TryFrom<&[u8]> for EspNowFrame {
    type Error = EspNowDecodingError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < ESPNOW_HEADER_SIZE {
            return Err(Self::Error::InvalidLength);
        }

        let mut decoded = EspNowFrame::default();

        decoded.mac_header.frame_control = bytes[0] as u16 | ((bytes[1] as u16) << 8);
        // TODO: Create custom type for frame control using bitfields
        let to_ds = decoded.mac_header.frame_control & (1 << 8) >> 8;
        let from_ds = decoded.mac_header.frame_control & (1 << 9) >> 9;

        if to_ds != 0 || from_ds != 0 {
            return Err(Self::Error::InvalidFrameControl);
        }

        decoded.mac_header.duration_or_id = bytes[2] as u16 | ((bytes[3] as u16) << 8);
        decoded.mac_header.destination =
            MacAddress::new(bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9]);
        decoded.mac_header.source = MacAddress::new(
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        );
        decoded.mac_header.broadcast = MacAddress::new(
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21],
        );

        if !decoded.mac_header.broadcast.is_broadcast() {
            return Err(Self::Error::InvalidBroadcastMAC);
        }

        decoded.mac_header.sequence_control = bytes[22] as u16 | ((bytes[23] as u16) << 8);
        decoded.category_code = bytes[24];

        if decoded.category_code != 127 {
            return Err(Self::Error::InvalidCategoryCode);
        }

        decoded.organization_identifier = [bytes[25], bytes[26], bytes[27]];

        const ESPRESSIF_ORGID: [u8; 3] = [0x18, 0xfe, 0x34];

        if decoded.organization_identifier != ESPRESSIF_ORGID {
            return Err(Self::Error::InvalidOrganizationIdentifier);
        }

        decoded.random = bytes[28] as u32
            | ((bytes[29] as u32) << 8)
            | ((bytes[30] as u32) << 16)
            | ((bytes[31] as u32) << 24);
        decoded.vendor_content.element_id = bytes[32];

        if decoded.vendor_content.element_id != 221 {
            return Err(Self::Error::InvalidElementId);
        }

        decoded.vendor_content.length = bytes[33];

        if decoded.vendor_content.length as usize != bytes.len() - 38 {
            return Err(Self::Error::InvalidLength);
        }

        decoded.vendor_content.organization_identifier = [bytes[34], bytes[35], bytes[36]];

        if decoded.vendor_content.organization_identifier != ESPRESSIF_ORGID {
            return Err(Self::Error::InvalidOrganizationIdentifier);
        }

        decoded.vendor_content.vc_type = bytes[37];
        const ESPNOW_TYPE: u8 = 4;

        if decoded.vendor_content.vc_type != ESPNOW_TYPE {
            return Err(Self::Error::InvalidVendorContentType);
        }

        decoded.vendor_content.version = bytes[38];
        let vc_start = ESPNOW_HEADER_SIZE;
        let vc_stop = vc_start + decoded.vendor_content.length as usize - 5;
        decoded.vendor_content.body = Vec::from(&bytes[vc_start..vc_stop]);

        let fcs_start = vc_stop;
        decoded.fcs = bytes[fcs_start] as u32
            | ((bytes[fcs_start + 1] as u32) << 8)
            | ((bytes[fcs_start + 2] as u32) << 16)
            | ((bytes[fcs_start + 3] as u32) << 24);

        Ok(decoded)
    }
}

impl EspNowFrame {
    pub fn get_body(&self) -> &[u8] {
        self.vendor_content.body.as_slice()
    }
}
