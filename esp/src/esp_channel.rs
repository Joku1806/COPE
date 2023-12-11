use core::ffi;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use cope::channel::{Channel, ChannelError};
use cope::config::CONFIG;
use cope::packet::Packet;
use cope_config::types::{mac_address::MacAddress, node_id::NodeID};

use esp_idf_svc::sys::{
    esp_wifi_set_promiscuous_rx_cb, wifi_promiscuous_pkt_type_t,
    wifi_promiscuous_pkt_type_t_WIFI_PKT_CTRL, wifi_promiscuous_pkt_type_t_WIFI_PKT_DATA,
    wifi_promiscuous_pkt_type_t_WIFI_PKT_MGMT, wifi_promiscuous_pkt_type_t_WIFI_PKT_MISC, EspError,
};
use esp_idf_svc::{
    espnow::{EspNow, PeerInfo, SendStatus},
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    sys::{esp, wifi_mode_t_WIFI_MODE_STA, wifi_second_chan_t_WIFI_SECOND_CHAN_NONE},
    wifi::{EspWifi, WifiDeviceId},
};

use crate::espnow_frame::EspNowFrame;
use crate::espressif_wifi_frame::EspressifWifiFrame;

pub struct EspChannel {
    // NOTE: We do not access the WiFi Driver after initialize(),
    // but we need to keep it around so it doesn't deinit when dropped.
    wifi_driver: EspWifi<'static>,
    espnow_driver: EspNow<'static>,
    own_mac: MacAddress,
    mac_map: HashMap<NodeID, MacAddress>,
    rx_queue: Arc<Mutex<VecDeque<Packet>>>,
}

enum PromiscuousPktType {
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
    Box<dyn FnMut(EspressifWifiFrame, PromiscuousPktType) -> Result<(), EspError> + 'static>,
> = None;

impl EspChannel {
    pub fn new(modem: Modem) -> Self {
        let sys_loop = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();
        let mut wifi_driver = EspWifi::new(modem, sys_loop, Some(nvs)).unwrap();
        wifi_driver.start().unwrap();
        let espnow_driver = EspNow::take().unwrap();
        let mac = MacAddress::from(wifi_driver.get_mac(WifiDeviceId::Sta).unwrap());

        return EspChannel {
            wifi_driver,
            espnow_driver,
            own_mac: mac,
            mac_map: HashMap::from(CONFIG.nodes),
            rx_queue: Arc::new(Mutex::new(VecDeque::new())),
        };
    }

    pub fn get_mac(&self) -> MacAddress {
        self.own_mac
    }

    unsafe extern "C" fn handle_promiscuous_rx(
        buf: *mut ffi::c_void,
        pkt_type: wifi_promiscuous_pkt_type_t,
    ) {
        // NOTE: There has to be a better way to do this.
        // It would be nice to just be able to transmute
        // from buf to a wifi_promiscuous_pkt_t, but that
        // is not possible because buf is not sized. The
        // internal representation in memory is probably
        // different as well.
        const HEADER_SIZE: usize = 48;
        let header = core::slice::from_raw_parts(buf as *mut u8, HEADER_SIZE);
        let payload_len = ((header[44] as u16) << 4) | (header[45] as u16 >> 4);
        let complete_buffer =
            core::slice::from_raw_parts(buf as *mut u8, HEADER_SIZE + payload_len as usize);

        let _ = PROMISCUOUS_RX_CALLBACK.as_mut().unwrap()(
            complete_buffer.try_into().unwrap(),
            pkt_type.try_into().unwrap(),
        );
    }

    fn set_promiscuous_rx_callback<'a, R>(&'a mut self, mut rx_callback: R) -> Result<(), EspError>
    where
        R: FnMut(EspressifWifiFrame, PromiscuousPktType) -> Result<(), EspError> + Send + 'static,
    {
        let _ = self.wifi_driver.disconnect();
        let _ = self.wifi_driver.stop();

        #[allow(clippy::type_complexity)]
        let rx_callback: Box<
            Box<
                dyn FnMut(EspressifWifiFrame, PromiscuousPktType) -> Result<(), EspError>
                    + Send
                    + 'a,
            >,
        > = Box::new(Box::new(move |frame, pkt_type| {
            rx_callback(frame, pkt_type)
        }));

        #[allow(clippy::type_complexity)]
        let rx_callback: Box<
            Box<
                dyn FnMut(EspressifWifiFrame, PromiscuousPktType) -> Result<(), EspError>
                    + Send
                    + 'static,
            >,
        > = unsafe { core::mem::transmute(rx_callback) };

        unsafe {
            PROMISCUOUS_RX_CALLBACK = Some(rx_callback);

            esp!(esp_wifi_set_promiscuous_rx_cb(Some(
                Self::handle_promiscuous_rx
            )))?;
        }

        Ok(())
    }

    pub fn initialize(&mut self) {
        let rx_queue_clone = self.rx_queue.clone();
        let rx_callback = move |wifi_frame: EspressifWifiFrame, _pkt_type: PromiscuousPktType| {
            if let Ok(espnow_frame) = EspNowFrame::try_from(wifi_frame.get_data()) {
                match Packet::deserialize_from(espnow_frame.get_body()) {
                    Ok(p) => rx_queue_clone.lock().unwrap().push_back(p),
                    Err(e) => log::warn!("Could not decode received packet: {}", e),
                };
            }

            Ok(())
        };

        self.set_promiscuous_rx_callback(rx_callback).unwrap();

        // NOTE: These are all init function calls I could find in the espnow examples
        // at https://github.com/espressif/esp-now/blob/master/examples.
        // Do we call all of these at some point?
        // esp_wifi_init(&cfg) - yes, in EspWifi::new
        // esp_wifi_set_mode(WIFI_MODE_STA) - yes, in wifi_driver.set_configuration
        // esp_wifi_set_storage(WIFI_STORAGE_RAM) - no, but NVS Flash is used as storage in EspDefaultNvsPartition::take
        // esp_wifi_set_ps(WIFI_PS_NONE) - yes, in EspNow::take
        // esp_wifi_start() - yes, in wifi_driver.start
        // espnow_init(&espnow_config); - yes, in EspNow::take
        unsafe {
            esp!(esp_idf_svc::sys::esp_wifi_set_mode(
                wifi_mode_t_WIFI_MODE_STA
            ))
            .unwrap();
            // NOTE: We need to be in promiscuous mode to overhear unicast packets
            // not addressed to us.
            esp!(esp_idf_svc::sys::esp_wifi_set_promiscuous(true)).unwrap();
        }
        self.wifi_driver.start().unwrap();
        unsafe {
            esp!(esp_idf_svc::sys::esp_wifi_set_channel(
                8,
                wifi_second_chan_t_WIFI_SECOND_CHAN_NONE
            ))
            .unwrap();
        }

        self.espnow_driver
            .register_send_cb(|mac: &[u8], status: SendStatus| {
                if matches!(status, SendStatus::FAIL) {
                    let fmt_mac = MacAddress::new(mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
                    log::warn!("Sending packet to {} failed!", fmt_mac);
                }
            })
            .unwrap();

        // NOTE: In unicast mode, the sender has to be paired with the receiver and vice versa.
        // To make sure this is guaranteed from the start, we add all unicast peers upfront.
        for (_, mac) in self.mac_map.iter() {
            if *mac != self.own_mac {
                self.add_unicast_peer(mac);
            }
        }
    }

    fn is_unicast_peer_added(&self, peer: &MacAddress) -> bool {
        return self.espnow_driver.peer_exists(peer.into_array()).unwrap();
    }

    fn add_unicast_peer(&self, peer: &MacAddress) {
        log::info!("Add unicast peer with MAC address {}", peer);

        let mut peer_info = PeerInfo::default();
        peer_info.peer_addr = peer.into_array();
        peer_info.channel = 0;
        peer_info.encrypt = false;
        peer_info.ifidx = esp_idf_svc::sys::wifi_interface_t_WIFI_IF_STA;

        self.espnow_driver.add_peer(peer_info).unwrap();
    }
}

impl Channel for EspChannel {
    fn transmit(&self, packet: &Packet) -> Result<(), ChannelError> {
        let receiver = match packet.canonical_receiver() {
            None => return Err(ChannelError::UnknownReceiver),
            Some(r) => r,
        };

        if let Some(mac) = self.mac_map.get(&receiver) {
            if !(self.is_unicast_peer_added(mac)) {
                log::warn!(
                    "Peer {} should have already been added. Is the peer part of the config?",
                    mac
                );
                self.add_unicast_peer(mac);
            }

            log::info!("Sending {:?} to {}", packet.coding_header(), mac);

            let serialized = packet.serialize_into().unwrap();
            // NOTE: How does backoff and ACKs work?
            // Does it happen automatically or do we have to write code for it?
            let result = self
                .espnow_driver
                .send(mac.into_array(), serialized.as_slice());

            return match result {
                Ok(_) => Ok(()),
                // FIXME: Figure out which esp_err_t codes map to our errors
                Err(_) => Err(ChannelError::NoACK),
            };
        }

        Ok(())
    }

    fn receive(&mut self) -> Option<Packet> {
        // NOTE: We need to combine back packets here,
        // once we allow them to be larger than the maximum EspNow Frame Size (250B).
        self.rx_queue.lock().unwrap().pop_front()
    }
}
