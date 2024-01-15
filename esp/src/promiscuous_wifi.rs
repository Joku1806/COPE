// TODO: Clean up error handling and try to upstream this to esp-idf-svc
use crate::wifi_frame::WifiFrame;
use bitvec::field::BitField;
use bitvec::prelude as bv;
use bitvec::view::BitView;
use core::ffi;
use esp_idf_svc::sys::{
    esp_wifi_set_promiscuous_rx_cb, wifi_promiscuous_pkt_type_t,
    wifi_promiscuous_pkt_type_t_WIFI_PKT_CTRL, wifi_promiscuous_pkt_type_t_WIFI_PKT_DATA,
    wifi_promiscuous_pkt_type_t_WIFI_PKT_MGMT, wifi_promiscuous_pkt_type_t_WIFI_PKT_MISC, EspError,
};
use esp_idf_svc::{sys::esp, wifi::EspWifi};
use std::error::Error;

#[derive(Debug)]
pub enum PromiscuousPktType {
    ManagementFrame,
    ControlFrame,
    DataFrame,
    MiscalleneousFrame,
}

impl From<PromiscuousPktType> for wifi_promiscuous_pkt_type_t {
    fn from(pkt_type: PromiscuousPktType) -> Self {
        match pkt_type {
            PromiscuousPktType::ManagementFrame => wifi_promiscuous_pkt_type_t_WIFI_PKT_MGMT,
            PromiscuousPktType::ControlFrame => wifi_promiscuous_pkt_type_t_WIFI_PKT_CTRL,
            PromiscuousPktType::DataFrame => wifi_promiscuous_pkt_type_t_WIFI_PKT_DATA,
            PromiscuousPktType::MiscalleneousFrame => wifi_promiscuous_pkt_type_t_WIFI_PKT_MISC,
        }
    }
}

impl From<wifi_promiscuous_pkt_type_t> for PromiscuousPktType {
    #[allow(non_upper_case_globals)]
    fn from(pkt_type: wifi_promiscuous_pkt_type_t) -> Self {
        match pkt_type {
            wifi_promiscuous_pkt_type_t_WIFI_PKT_MGMT => PromiscuousPktType::ManagementFrame,
            wifi_promiscuous_pkt_type_t_WIFI_PKT_CTRL => PromiscuousPktType::ControlFrame,
            wifi_promiscuous_pkt_type_t_WIFI_PKT_DATA => PromiscuousPktType::DataFrame,
            wifi_promiscuous_pkt_type_t_WIFI_PKT_MISC => PromiscuousPktType::MiscalleneousFrame,
            _ => panic!(),
        }
    }
}

#[allow(clippy::type_complexity)]
static mut PROMISCUOUS_RX_CALLBACK: Option<
    Box<dyn FnMut(WifiFrame, PromiscuousPktType) -> Result<(), Box<dyn Error>> + 'static>,
> = None;

unsafe extern "C" fn handle_promiscuous_rx(
    buf: *mut ffi::c_void,
    pkt_type: wifi_promiscuous_pkt_type_t,
) {
    // NOTE: There has to be a better way to do this. It would be nice to just be
    // able to transmute from buf to a wifi_promiscuous_pkt_t, but that is not
    // possible because buf is not sized. The internal representation in memory is
    // probably different as well.
    // FIXME: When upstreaming, we should not use the bitvec crate to do this. Just
    // do some bitshifting magic.
    const HEADER_SIZE: usize = 48;
    let header = core::slice::from_raw_parts(buf as *mut u8, HEADER_SIZE);
    let bits = header.view_bits::<bv::Lsb0>();
    let sig_len = bits[352..364].load::<usize>();
    let complete_buffer = core::slice::from_raw_parts(buf as *mut u8, HEADER_SIZE + sig_len);

    let rs_cb = match PROMISCUOUS_RX_CALLBACK.as_mut() {
        Some(cb) => cb,
        None => {
            log::warn!("No promiscuous WiFi callback registered!");
            return;
        }
    };

    let rs_frame: WifiFrame = match complete_buffer.try_into() {
        Ok(f) => f,
        Err(e) => {
            log::warn!("Not a valid WiFi frame: {:?}", e);
            return;
        }
    };

    let rs_pkt_type = match pkt_type.try_into() {
        Ok(pt) => pt,
        Err(e) => {
            log::warn!("Not a valid packet type: {:?}", e);
            return;
        }
    };

    if let Err(e) = rs_cb(rs_frame, rs_pkt_type) {
        log::warn!("Reception error: {}", e);
    }
}

pub fn set_promiscuous_rx_callback<'a, R>(
    wifi_driver: &'a mut EspWifi,
    mut rx_callback: R,
) -> Result<(), EspError>
where
    R: FnMut(WifiFrame, PromiscuousPktType) -> Result<(), Box<dyn Error>> + Send + 'static,
{
    let _ = wifi_driver.disconnect();
    let _ = wifi_driver.stop();

    #[allow(clippy::type_complexity)]
    let rx_callback: Box<
        Box<dyn FnMut(WifiFrame, PromiscuousPktType) -> Result<(), Box<dyn Error>> + Send + 'a>,
    > = Box::new(Box::new(move |frame, pkt_type| {
        rx_callback(frame, pkt_type)
    }));

    #[allow(clippy::type_complexity)]
    let rx_callback: Box<
        Box<
            dyn FnMut(WifiFrame, PromiscuousPktType) -> Result<(), Box<dyn Error>> + Send + 'static,
        >,
    > = unsafe { core::mem::transmute(rx_callback) };

    unsafe {
        PROMISCUOUS_RX_CALLBACK = Some(rx_callback);

        esp!(esp_wifi_set_promiscuous_rx_cb(Some(handle_promiscuous_rx)))?;
    }

    Ok(())
}
