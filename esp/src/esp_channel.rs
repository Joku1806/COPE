use std::collections::{HashMap, VecDeque};

use esp_idf_svc::espnow::{EspNow, PeerInfo};

use cope::channel::{Channel, ChannelError};
use cope::packet::Packet;
use cope::topology::NodeID;

type MacAddress = [u8; 6];

pub struct EspChannel<'a> {
    espnow_driver: EspNow<'a>,
    // TODO: Get this from config
    mac_map: HashMap<NodeID, MacAddress>,
    received_packets: VecDeque<Packet>,
}

impl EspChannel<'_> {
    pub fn new() -> Self {
        let espnow_driver = EspNow::take().unwrap();

        return EspChannel {
            espnow_driver,
            // TODO: get from config
            mac_map: HashMap::new(),
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
                self.received_packets
                    .push_back(Packet::deserialize_from(bytes).unwrap());
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
        return self.espnow_driver.peer_exists(*peer).unwrap();
    }

    fn add_unicast_peer(&self, peer: &MacAddress) {
        let mut peer_info = PeerInfo::default();
        peer_info.peer_addr = *peer;
        peer_info.channel = 0;
        peer_info.encrypt = false;

        // TODO: Better error handling
        self.espnow_driver.add_peer(peer_info).unwrap();
    }
}

impl Channel for EspChannel<'_> {
    fn transmit(&self, packet: &Packet) -> Result<(), ChannelError> {
        if let Some(mac) = self.mac_map.get(&packet.get_sender()) {
            if !self.is_unicast_peer_added(mac) {
                self.add_unicast_peer(mac);
            }

            let serialized = packet.serialize_into().unwrap();
            // NOTE: How does backoff and ACKs work?
            // Does it happen automatically or do we have to write code for it?
            let result = self.espnow_driver.send(*mac, serialized.as_slice());

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
