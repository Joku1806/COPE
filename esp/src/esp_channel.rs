use crate::espnow_frame::{EspNowDecodingError, EspNowFrame};
use crate::frame_collection::{Frame, FrameCollection, FrameCollectionError, FrameError};
use crate::promiscuous_wifi;
use crate::wifi_frame::WifiFrame;
use cope::channel::Channel;
use cope::config::CONFIG;
use cope::packet::Packet;
use cope::stats::StatsLogger;
use cope_config::types::{mac_address::MacAddress, node_id::NodeID};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::{
    espnow::{EspNow, PeerInfo, SendStatus},
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    sys::{
        esp, esp_wifi_set_promiscuous_filter, wifi_mode_t_WIFI_MODE_STA, wifi_promiscuous_filter_t,
        wifi_second_chan_t_WIFI_SECOND_CHAN_NONE, WIFI_PROMIS_FILTER_MASK_MGMT,
    },
    wifi::{EspWifi, WifiDeviceId},
};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

const RX_QUEUE_MAX_SIZE: usize = 128;
const ESPNOW_FRAME_SIZE: u8 = 250;

#[derive(Debug)]
pub enum EspChannelError {
    UnknownReceiver,
    UnicastPeerError(MacAddress, EspError),
    SerializationError(bincode::Error),
    FrameEncodingError(FrameCollectionError),
    FrameDecodingError(FrameError),
    // FIXME: Stupid name, try to find a better one
    PacketDecodingError(FrameCollectionError),
    _EspNowFrameDecodingError(EspNowDecodingError),
    EspNowTransmissionError(EspError),
    EspNowTransmissionCallbackError(MacAddress),
}

impl std::fmt::Display for EspChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EspChannelError::UnknownReceiver => write!(f, "packet receiver was not set"),
            EspChannelError::UnicastPeerError(mac, e) => {
                write!(f, "could not add {} as peer: {}", mac, e)
            }
            EspChannelError::SerializationError(e) => {
                write!(f, "could not serialize packet: {}", e)
            }
            EspChannelError::FrameEncodingError(e) => {
                // FIXME: Implement Display trait for FrameCollectionError
                write!(
                    f,
                    "could not encode serialized packet into partial packet frames: {:?}",
                    e
                )
            }
            EspChannelError::FrameDecodingError(e) => {
                // FIXME: Implement Display trait for FrameError
                write!(f, "could not decode partial packet frame: {:?}", e)
            }
            EspChannelError::PacketDecodingError(e) => {
                // FIXME: Implement Display trait for FrameCollectionError
                write!(
                    f,
                    "could not add partial packet frame to collection: {:?}",
                    e
                )
            }
            EspChannelError::_EspNowFrameDecodingError(e) => {
                // FIXME: Implement Display trait for EspNowDecodingError
                write!(f, "could not decode bytes into EspNow frame: {:?}", e)
            }
            EspChannelError::EspNowTransmissionError(e) => {
                write!(f, "could not transmit EspNow frame: {}", e)
            }
            EspChannelError::EspNowTransmissionCallbackError(mac) => {
                write!(f, "could not transmit EspNow frame to {}", mac)
            }
        }
    }
}

impl Error for EspChannelError {}

pub struct EspStatsLogger {
    path: String,
}

impl StatsLogger for EspStatsLogger {
    fn new(path: &str) -> Result<Self, std::io::Error> {
        Ok(Self {
            path: path.to_owned(),
        })
    }

    fn log(&mut self, data: &str) {
        println!("STATS {} {}", self.path, data);
    }
}

pub struct EspChannel {
    // NOTE: We do not access the WiFi Driver after initialize(), but we need to keep it around so
    // it doesn't deinit when dropped.
    wifi_driver: EspWifi<'static>,
    espnow_driver: EspNow<'static>,
    own_mac: MacAddress,
    mac_map: HashMap<NodeID, MacAddress>,
    rx_buffer: Arc<Mutex<HashMap<u32, (SystemTime, FrameCollection)>>>,
    tx_callback_done: Arc<Mutex<bool>>,
    tx_callback_result: Arc<Mutex<Result<(), EspChannelError>>>,
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
            tx_callback_result: Arc::new(Mutex::new(Ok(()))),
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
            // NOTE: Do we want to sniff ACKs here as well or do we handle that all through
            // the COPE protocol ACKs at a higher abstraction level?
            let filter = wifi_promiscuous_filter_t {
                filter_mask: WIFI_PROMIS_FILTER_MASK_MGMT,
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
        let rx_callback = move |wifi_frame: WifiFrame,
                                pkt_type: promiscuous_wifi::PromiscuousPktType|
              -> Result<(), Box<dyn Error>> {
            let espnow_frame = match EspNowFrame::try_from(wifi_frame.get_data()) {
                Ok(f) => f,
                // NOTE: The WiFi sniffer also receives other data that is not EspNow. To not
                // spam the debug output, we silence this warning.
                Err(_e) => return Ok(()),
            };

            log::info!(
                "Received an EspNow frame {:?} with packet type {:?}",
                espnow_frame,
                pkt_type
            );

            let partial_frame = match Frame::try_from(espnow_frame.get_body()) {
                Ok(f) => f,
                Err(e) => return Err(Box::new(EspChannelError::FrameDecodingError(e))),
            };

            let mut buffer = rx_buffer_clone.lock().unwrap();

            if !buffer.contains_key(&partial_frame.get_magic()) {
                if buffer.len() >= RX_QUEUE_MAX_SIZE {
                    log::warn!("Have to drop packet, because RX queue is full!");
                    return Ok(());
                }

                buffer.insert(
                    partial_frame.get_magic(),
                    (SystemTime::now(), FrameCollection::new()),
                );
            }

            let frame_collection = buffer.get_mut(&partial_frame.get_magic()).unwrap();

            if let Err(e) = frame_collection.1.add_frame(partial_frame) {
                return Err(Box::new(EspChannelError::PacketDecodingError(e)));
            }

            // NOTE: We reset the collection timestamp here, since a frame indicates that
            // the packet is still in transit. If we would only set the timestamp for the
            // first frame, we could prematurely drop large packets or packets sent during
            // high congestion.
            frame_collection.0 = SystemTime::now();

            Ok(())
        };

        promiscuous_wifi::set_promiscuous_rx_callback(&mut self.wifi_driver, rx_callback)?;

        let tx_callback_done_clone = self.tx_callback_done.clone();
        let tx_result_clone = self.tx_callback_result.clone();
        self.espnow_driver
            .register_send_cb(move |mac: &[u8], status: SendStatus| {
                // NOTE: It is not possible for the send callback to return an error, so we have
                // to log them here, which is kind of annoying.
                if matches!(status, SendStatus::FAIL) {
                    *tx_result_clone.lock().unwrap() =
                        Err(EspChannelError::EspNowTransmissionCallbackError(
                            MacAddress::new(mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]),
                        ));
                }

                *tx_callback_done_clone.lock().unwrap() = true;
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
}

impl Channel for EspChannel {
    fn transmit(&mut self, packet: &Packet) -> Result<(), Box<dyn Error>> {
        let receiver = match packet.canonical_receiver() {
            None => return Err(Box::new(EspChannelError::UnknownReceiver)),
            Some(r) => r,
        };

        if let Some(mac) = self.mac_map.get(&receiver) {
            if !(self.is_unicast_peer_added(mac)) {
                log::warn!(
                    "Peer {} should have already been added. Is the peer part of the config?",
                    mac
                );

                if let Err(e) = self.add_unicast_peer(mac) {
                    return Err(Box::new(EspChannelError::UnicastPeerError(*mac, e)));
                }
            }

            log::info!("Sending {:?} to {}", packet, mac);

            let serialized = match packet.serialize_into() {
                Ok(s) => s,
                Err(e) => return Err(Box::new(EspChannelError::SerializationError(e))),
            };

            log::info!(
                "Serialized packet: {:?} ({} Bytes)",
                serialized,
                serialized.len()
            );

            let mut frames = match FrameCollection::new().with_frame_size(ESPNOW_FRAME_SIZE) {
                Ok(fc) => fc,
                Err(e) => return Err(Box::new(EspChannelError::FrameEncodingError(e))),
            };

            if let Err(e) = frames.encode(serialized.as_slice()) {
                return Err(Box::new(EspChannelError::FrameEncodingError(e)));
            }

            log::info!("Encoded as Frames: {:?}", frames);

            for frame in frames.iter() {
                // TODO: Make this work without clone
                let frame_serialized: Vec<u8> = frame.clone().unwrap().into();
                log::info!("Transmitting: {:?}", frame_serialized);

                let result = self
                    .espnow_driver
                    .send(mac.into_array(), frame_serialized.as_slice());

                // NOTE: We wait here until the TX callback ran and reset this flag. This is
                // recommended practice in the esp-idf espnow guide. Apparently transmissions
                // can fail if you send too quickly.
                while !*self.tx_callback_done.lock().unwrap() {}

                if result.is_err() {
                    // TODO: Should we return an error here? We have not sent the other frames yet.
                    return Err(Box::new(EspChannelError::EspNowTransmissionError(
                        result.err().unwrap(),
                    )));
                }

                let mut tx_callback_result = self.tx_callback_result.lock().unwrap();
                if tx_callback_result.is_err() {
                    // TODO: There has to be a better way to do this in Rust.
                    // This is the ugliest code I have ever written.
                    let mac = match tx_callback_result.as_ref().unwrap_err() {
                        EspChannelError::EspNowTransmissionCallbackError(mac) => *mac,
                        _ => MacAddress::new(0, 0, 0, 0, 0, 0),
                    };

                    let copy: Result<(), Box<dyn std::error::Error>> = Err(Box::new(
                        EspChannelError::EspNowTransmissionCallbackError(mac),
                    ));

                    *tx_callback_result = Ok(());

                    return copy;
                }
            }
        }

        Ok(())
    }

    fn receive(&mut self) -> Option<Packet> {
        let mut packet = None;

        self.rx_buffer
            .lock()
            .unwrap()
            .retain(|_, (last_frame_rx, collection)| {
                let elapsed = match last_frame_rx.elapsed() {
                    Ok(elapsed) => elapsed,
                    Err(_) => Duration::ZERO,
                };

                // NOTE: Any incomplete packets, where the last frame was received RTT
                // ago are assumed to be lost and should be removed.
                if elapsed >= CONFIG.round_trip_time && !collection.is_complete() {
                    return false;
                }

                // NOTE: Non-decodable packets, as well as the packet which is returned should
                // be removed.
                if collection.is_complete() && packet.is_none() {
                    packet = collection
                        .decode()
                        .map_err(|e| log::warn!("Could not decode frame collection: {:?}", e))
                        .ok()
                        .and_then(|bytes| {
                            Packet::deserialize_from(bytes.as_slice())
                                .map_err(|e| log::warn!("Could not decode packet: {}", e))
                                .ok()
                        });

                    return false;
                }

                return true;
            });

        // NOTE: free memory after cleaning out map
        self.rx_buffer.lock().unwrap().shrink_to_fit();

        log::info!(
            "RX queue has {} packets ready for dequeing",
            self.rx_buffer.lock().unwrap().len()
        );

        packet
    }
}
