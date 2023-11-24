use std::collections::{HashMap, VecDeque};

use esp_idf_svc::espnow::{EspNow, PeerInfo};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

use cope::channel::Channel;
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
        // TODO: Figure out how to move this to initialize()
        let peripherals = Peripherals::take().unwrap();
        let sys_loop = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();

        // TODO: Better error handling
        let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();
        wifi_driver.start().unwrap();
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
    fn transmit(&self, packet: &Packet) {
        if let Some(mac) = self.mac_map.get(&packet.get_sender()) {
            if !self.is_unicast_peer_added(mac) {
                self.add_unicast_peer(mac);
            }

            // TODO: actually send packet
            self.espnow_driver
                .send(*mac, "Placeholder".as_bytes())
                .unwrap();
        }
    }

    fn receive(&mut self) -> Option<Packet> {
        return self.received_packets.pop_back();
    }
}
