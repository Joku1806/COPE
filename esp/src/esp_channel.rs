use std::collections::{HashMap, VecDeque};

use cope::config::CONFIG;
use cope_config::types::mac_address::MacAddress;
use cope_config::types::node_id::NodeID;
use esp_idf_svc::espnow::{EspNow, PeerInfo};

use cope::channel::{Channel, ChannelError};
use cope::packet::Packet;

use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi, WifiDeviceId},
};

pub struct EspChannel<'a> {
    espnow_driver: EspNow<'a>,
    own_mac: MacAddress,
    mac_map: HashMap<NodeID, MacAddress>,
    received_packets: VecDeque<Packet>,
}

impl EspChannel<'_> {
    pub fn new(modem: Modem) -> Self {
        let sys_loop = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();

        let mut wifi_driver = EspWifi::new(modem, sys_loop, Some(nvs)).unwrap();
        wifi_driver.start().unwrap();
        wifi_driver
            .set_configuration(&Configuration::Client(ClientConfiguration {
                ssid: "".into(),
                bssid: None,
                auth_method: AuthMethod::None,
                password: "".into(),
                channel: Some(8),
            }))
            .unwrap();
        // NOTE: We need to be in promiscuous mode to overhear unicast packets
        // not addressed to us.
        unsafe {
            esp_idf_svc::sys::esp_wifi_set_promiscuous(true);
        }

        let espnow_driver = EspNow::take().unwrap();
        let mac = MacAddress::from(wifi_driver.get_mac(WifiDeviceId::Sta).unwrap());

        return EspChannel {
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
                // FIXME: Do not panic if we get a malformed packet
                let deserialized = Packet::deserialize_from(bytes);
                if let Err(e) = deserialized {
                    log::warn!("Could not decode received packet: {}", e);
                    return;
                }

                self.received_packets.push_back(deserialized.unwrap());
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

        self.espnow_driver.add_peer(peer_info).unwrap();
    }
}

impl Channel for EspChannel<'_> {
    fn transmit(&self, packet: &Packet) -> Result<(), ChannelError> {
        if let Some(mac) = self.mac_map.get(&packet.sender()) {
            assert!(self.is_unicast_peer_added(mac));

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
        return self.received_packets.pop_front();
    }
}
