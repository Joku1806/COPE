use std::collections::{HashMap, VecDeque};

use cope::config::CONFIG;
use cope_config::types::mac_address::MacAddress;
use cope_config::types::node_id::NodeID;
use esp_idf_svc::espnow::{EspNow, PeerInfo};

use cope::channel::{Channel, ChannelError};
use cope::packet::Packet;

use log::info;

pub struct EspChannel<'a> {
    espnow_driver: EspNow<'a>,
    mac_map: HashMap<NodeID, MacAddress>,
    received_packets: VecDeque<Packet>,
}

impl EspChannel<'_> {
    pub fn new() -> Self {
        let espnow_driver = EspNow::take().unwrap();

        return EspChannel {
            espnow_driver,
            mac_map: HashMap::from(CONFIG.nodes),
            received_packets: VecDeque::new(),
        };
    }

    // NOTE: Make required function for Channel trait?
    pub fn initialize(&mut self) {
        // TODO: Find out what the two parameters of the callback function are.
        // I'm not sure the current interpretation is correct,
        // I just compared with the definition of esp_now_recv_cb_t
        self.espnow_driver
            .register_recv_cb(|_info: &[u8], bytes: &[u8]| {
                // FIXME: Do not panic if we get a malformed packet
                self.received_packets
                    .push_back(Packet::deserialize_from(bytes).unwrap());
                log::info!("[ESPChannel] Received an ESPNow packet.");
            })
            .unwrap();

        unsafe {
            // NOTE: We need to be in promiscuous mode to overhear unicast packets
            // not addressed to us.
            // TODO: Error handling?
            esp_idf_svc::sys::esp_wifi_set_promiscuous(true);
        }
    }

    fn is_unicast_peer_added(&self, peer: &MacAddress) -> bool {
        // TODO: Better error handling
        return self.espnow_driver.peer_exists(peer.into_array()).unwrap();
    }

    fn add_unicast_peer(&self, peer: &MacAddress) {
        log::info!("[ESPChannel] Add unicast peer with MAC address {}", peer);

        let mut peer_info = PeerInfo::default();
        peer_info.peer_addr = peer.into_array();
        peer_info.channel = 0;
        peer_info.encrypt = false;

        // TODO: Better error handling
        self.espnow_driver.add_peer(peer_info).unwrap();
    }
}

impl Channel for EspChannel<'_> {
    fn transmit(&self, packet: &Packet) -> Result<(), ChannelError> {
        if let Some(mac) = self.mac_map.get(&packet.sender()) {
            if !self.is_unicast_peer_added(mac) {
                self.add_unicast_peer(mac);
            }

            log::info!(
                "[ESPChannel] Sending {:?} to {}",
                packet.coding_header(),
                mac
            );

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
