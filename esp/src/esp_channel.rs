use core::hint::black_box;
use std::collections::{HashMap, VecDeque};

use cope::config::CONFIG;
use cope_config::types::mac_address::MacAddress;
use cope_config::types::node_id::NodeID;
use esp_idf_svc::espnow::{EspNow, PeerInfo};

use cope::channel::{Channel, ChannelError};
use cope::packet::Packet;

use esp_idf_svc::{
    espnow::SendStatus,
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    sys::{esp, wifi_mode_t_WIFI_MODE_STA, wifi_second_chan_t_WIFI_SECOND_CHAN_NONE},
    wifi::{EspWifi, WifiDeviceId},
};

pub struct EspChannel<'a> {
    // NOTE: We do not access the WiFi Driver after initialize(),
    // but we need to keep it around so it doesn't deinit when dropped.
    _wifi_driver: EspWifi<'a>,
    espnow_driver: EspNow<'a>,
    own_mac: MacAddress,
    mac_map: HashMap<NodeID, MacAddress>,
    received_packets: VecDeque<Packet>,
}

impl EspChannel<'_> {
    pub fn new(modem: Modem) -> Self {
        let sys_loop = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();

        // NOTE: These are all init function calls I could find in the espnow examples
        // at https://github.com/espressif/esp-now/blob/master/examples.
        // Do we call all of these at some point?
        // esp_wifi_init(&cfg) - yes, in EspWifi::new
        // esp_wifi_set_mode(WIFI_MODE_STA) - yes, in wifi_driver.set_configuration
        // esp_wifi_set_storage(WIFI_STORAGE_RAM) - no, but NVS Flash is used as storage in EspDefaultNvsPartition::take
        // esp_wifi_set_ps(WIFI_PS_NONE) - yes, in EspNow::take
        // esp_wifi_start() - yes, in wifi_driver.start
        // espnow_init(&espnow_config); - yes, in EspNow::take
        let mut wifi_driver = EspWifi::new(modem, sys_loop, Some(nvs)).unwrap();
        unsafe {
            esp!(esp_idf_svc::sys::esp_wifi_set_mode(
                wifi_mode_t_WIFI_MODE_STA
            ))
            .unwrap();
            // NOTE: We need to be in promiscuous mode to overhear unicast packets
            // not addressed to us.
            esp!(esp_idf_svc::sys::esp_wifi_set_promiscuous(true)).unwrap();
        }
        wifi_driver.start().unwrap();
        unsafe {
            esp!(esp_idf_svc::sys::esp_wifi_set_channel(
                8,
                wifi_second_chan_t_WIFI_SECOND_CHAN_NONE
            ))
            .unwrap();
        }

        let espnow_driver = EspNow::take().unwrap();
        let mac = MacAddress::from(wifi_driver.get_mac(WifiDeviceId::Sta).unwrap());

        return EspChannel {
            _wifi_driver: wifi_driver,
            espnow_driver,
            own_mac: mac,
            mac_map: HashMap::from(CONFIG.nodes),
            received_packets: VecDeque::new(),
        };
    }

    pub fn get_mac(&self) -> MacAddress {
        self.own_mac
    }

    pub fn initialize(&mut self) {
        // TODO: Find out what the two parameters of the callback function are.
        // I'm not sure the current interpretation is correct,
        // I just compared with the definition of esp_now_recv_cb_t
        self.espnow_driver
            .register_recv_cb(|_info: &[u8], bytes: &[u8]| {
                match Packet::deserialize_from(bytes) {
                    Ok(p) => self.received_packets.push_back(p),
                    Err(e) => log::warn!("Could not decode received packet: {}", e),
                };
            })
            .unwrap();

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

impl Channel for EspChannel<'_> {
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
        return black_box(self.received_packets.pop_front());
    }
}
