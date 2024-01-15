use crate::espnow_frame::EspNowFrame;
use crate::frame_collection::{Frame, FrameCollection};
use crate::promiscuous_wifi;
use crate::wifi_frame::WifiFrame;
use cope::channel::{Channel, ChannelError};
use cope::config::CONFIG;
use cope::packet::Packet;
use cope_config::types::{mac_address::MacAddress, node_id::NodeID};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::{
    espnow::{EspNow, PeerInfo, SendStatus},
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    sys::{
        esp, esp_wifi_set_promiscuous_filter, wifi_mode_t_WIFI_MODE_STA, wifi_promiscuous_filter_t,
        wifi_second_chan_t_WIFI_SECOND_CHAN_NONE, WIFI_PROMIS_FILTER_MASK_ALL,
    },
    wifi::{EspWifi, WifiDeviceId},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// TODO: Make settable from config
const RX_DRAIN_TIME: Duration = Duration::from_millis(500);
const ESPNOW_FRAME_SIZE: u8 = 250;

pub struct EspChannel {
    // NOTE: We do not access the WiFi Driver after initialize(), but we need to keep it around so
    // it doesn't deinit when dropped.
    wifi_driver: EspWifi<'static>,
    espnow_driver: EspNow<'static>,
    own_mac: MacAddress,
    mac_map: HashMap<NodeID, MacAddress>,
    rx_buffer: Arc<Mutex<HashMap<u32, (SystemTime, FrameCollection)>>>,
    tx_callback_done: Arc<Mutex<bool>>,
}

impl EspChannel {
    pub fn new(modem: Modem) -> Result<Self, EspError> {
        let sys_loop = EspSystemEventLoop::take()?;
        let nvs = EspDefaultNvsPartition::take()?;
        let mut wifi_driver = EspWifi::new(modem, sys_loop, Some(nvs))?;
        wifi_driver.start()?;
        let espnow_driver = EspNow::take()?;
        let mac = MacAddress::from(wifi_driver.get_mac(WifiDeviceId::Sta)?);

        Ok(EspChannel {
            wifi_driver,
            espnow_driver,
            own_mac: mac,
            mac_map: HashMap::from(CONFIG.nodes),
            rx_buffer: Arc::new(Mutex::new(HashMap::new())),
            tx_callback_done: Arc::new(Mutex::new(false)),
        })
    }

    fn set_wifi_config_and_start(&mut self) -> Result<(), EspError> {
        // NOTE: These are all init function calls I could find in the espnow examples
        // at https://github.com/espressif/esp-now/blob/master/examples.
        // Do we call all of these at some point?
        // esp_wifi_init(&cfg) - yes, in EspWifi::new
        // esp_wifi_set_mode(WIFI_MODE_STA) - yes, in wifi_driver.set_configuration
        // esp_wifi_set_storage(WIFI_STORAGE_RAM) - no, but NVS Flash is used as storage
        // in EspDefaultNvsPartition::take esp_wifi_set_ps(WIFI_PS_NONE) - yes,
        // in EspNow::take esp_wifi_start() - yes, in wifi_driver.start
        // espnow_init(&espnow_config); - yes, in EspNow::take
        unsafe {
            esp!(esp_idf_svc::sys::esp_wifi_set_mode(
                wifi_mode_t_WIFI_MODE_STA
            ))?;
            // NOTE: We need to be in promiscuous mode to overhear unicast packets
            // not addressed to us.
            esp!(esp_idf_svc::sys::esp_wifi_set_promiscuous(true))?;
            // FIXME: Find out if EspNow frames are always received as a specific
            // PromiscuousPktType in promiscuous mode. This would allow us to
            // throw away all other frames more quickly.
            let filter = wifi_promiscuous_filter_t {
                filter_mask: WIFI_PROMIS_FILTER_MASK_ALL,
            };
            esp!(esp_wifi_set_promiscuous_filter(&filter))?;
        }
        self.wifi_driver.start()?;
        unsafe {
            // TODO: Make settable through config
            esp!(esp_idf_svc::sys::esp_wifi_set_channel(
                8,
                wifi_second_chan_t_WIFI_SECOND_CHAN_NONE
            ))?;
        }

        Ok(())
    }

    fn register_callbacks(&mut self) -> Result<(), EspError> {
        let rx_buffer_clone = self.rx_buffer.clone();
        let rx_callback =
            move |wifi_frame: WifiFrame, _pkt_type: promiscuous_wifi::PromiscuousPktType| {
                // TODO: Better error handling!
                let espnow_frame = match EspNowFrame::try_from(wifi_frame.get_data()) {
                    Ok(f) => f,
                    Err(_) => return Ok(()),
                };

                let partial_frame = match Frame::try_from(espnow_frame.get_body()) {
                    Ok(f) => f,
                    Err(_) => return Ok(()),
                };

                let mut buffer = rx_buffer_clone.lock().unwrap();

                if !buffer.contains_key(&partial_frame.get_magic()) {
                    buffer.insert(
                        partial_frame.get_magic(),
                        (SystemTime::now(), FrameCollection::new()),
                    );
                }

                let frame_collection = buffer.get_mut(&partial_frame.get_magic()).unwrap();

                if let Err(e) = frame_collection.1.add_frame(partial_frame) {
                    log::warn!("Could not process received partial frame: {:?}", e);
                }

                Ok(())
            };

        promiscuous_wifi::set_promiscuous_rx_callback(&mut self.wifi_driver, rx_callback)?;

        let tx_callback_done_clone = self.tx_callback_done.clone();
        self.espnow_driver
            .register_send_cb(move |mac: &[u8], status: SendStatus| {
                if matches!(status, SendStatus::FAIL) {
                    let fmt_mac = MacAddress::new(mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
                    log::warn!("Sending packet to {} failed!", fmt_mac);
                }

                let mut done = tx_callback_done_clone.lock().unwrap();

                if *done {
                    log::warn!(
                        "TX Callback status flag was not reset after previous transmission!"
                    );
                }

                *done = true;
            })?;

        Ok(())
    }

    pub fn initialize(&mut self) -> Result<(), EspError> {
        self.register_callbacks()?;
        self.set_wifi_config_and_start()?;

        // NOTE: In unicast mode, the sender has to be paired with the receiver and vice
        // versa. To make sure this is guaranteed from the start, we add all unicast
        // peers upfront. FIXME: There is a hard limit of 20 concurrent Unicast peers.
        // If we need more than that in the future, we need to add and remove them
        // dynamically somehow. This will be complicated, since messages can only be
        // received correctly, if both sender and receiver have each other added as
        // peers.
        for (_, mac) in self.mac_map.iter() {
            if *mac != self.own_mac {
                self.add_unicast_peer(mac)?;
            }
        }

        Ok(())
    }

    fn is_unicast_peer_added(&self, peer: &MacAddress) -> bool {
        match self.espnow_driver.peer_exists(peer.into_array()) {
            Ok(res) => res,
            Err(_) => false,
        }
    }

    fn add_unicast_peer(&self, peer: &MacAddress) -> Result<(), EspError> {
        log::info!("Add unicast peer with MAC address {}", peer);

        let mut peer_info = PeerInfo::default();
        peer_info.peer_addr = peer.into_array();
        peer_info.channel = 0;
        peer_info.encrypt = false;
        peer_info.ifidx = esp_idf_svc::sys::wifi_interface_t_WIFI_IF_STA;

        self.espnow_driver.add_peer(peer_info)
    }

    pub fn get_mac(&self) -> MacAddress {
        self.own_mac
    }

    fn collect_packet(&mut self) -> Option<Packet> {
        for (_, (creation_time, collection)) in self.rx_buffer.lock().unwrap().iter() {
            if collection.is_complete() {
                if let Ok(elapsed) = creation_time.elapsed() {
                    if elapsed > RX_DRAIN_TIME {
                        log::warn!("RX Drain Time is set too low.");
                    }
                }

                // TODO: make this work without clone
                match collection.decode() {
                    Ok(bytes) => match Packet::deserialize_from(bytes.as_slice()) {
                        Ok(p) => return Some(p),
                        Err(e) => log::warn!("Could not decode received packet: {}", e),
                    },
                    Err(e) => log::warn!("Could not piece together frame collection: {:?}", e),
                };
            }
        }

        None
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

                if let Err(e) = self.add_unicast_peer(mac) {
                    log::warn!("Could not add peer: {}", e);
                    // TODO: real error handling
                    return Err(ChannelError::NoACK);
                }
            }

            log::info!("Sending {:?} to {}", packet.coding_header(), mac);

            let serialized = match packet.serialize_into() {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Could not serialize packet: {}", e);
                    // TODO: real error handling
                    return Err(ChannelError::NoACK);
                }
            };

            let mut frames = match FrameCollection::new().with_frame_size(ESPNOW_FRAME_SIZE) {
                Ok(fc) => fc,
                Err(e) => {
                    log::warn!("Error while creating FrameCollection: {:?}", e);
                    // TODO: Real error handling
                    return Err(ChannelError::NoACK);
                }
            };

            if let Err(e) = frames.encode(serialized.as_slice()) {
                log::warn!("Error while enconding packet: {:?}", e);
                // TODO: Real error handling
                return Err(ChannelError::NoACK);
            }

            for frame in frames.iter() {
                // TODO: Make this work without clone
                let serialized: Vec<u8> = frame.clone().unwrap().into();
                let result = self
                    .espnow_driver
                    .send(mac.into_array(), serialized.as_slice());

                // NOTE: We wait here until the TX callback ran and reset this flag. This is
                // recommended practice in the esp-idf espnow guide. Apparently transmissions
                // can fail if you send too quickly.
                while !*self.tx_callback_done.lock().unwrap() {}

                if result.is_err() {
                    // TODO: Real error handling. Specifically here, do we return an error? We have
                    // not sent the other frames yet.
                    return Err(ChannelError::NoACK);
                }
            }
        }

        Ok(())
    }

    fn receive(&mut self) -> Option<Packet> {
        self.rx_buffer
            .lock()
            .unwrap()
            .retain(|_, (creation_time, collection)| {
                let elapsed = match creation_time.elapsed() {
                    Ok(elapsed) => elapsed,
                    Err(_) => Duration::ZERO,
                };
                elapsed < RX_DRAIN_TIME || collection.is_complete()
            });

        self.collect_packet()
    }
}
