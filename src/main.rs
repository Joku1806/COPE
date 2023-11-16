#![feature(custom_test_frameworks)]
#![test_runner(test_runner::run)]

mod packet;
mod test_runner;

use esp_idf_svc::espnow::{EspNow, PeerInfo, BROADCAST};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use std::{thread::sleep, time::Duration};

use crate::packet::{Packet, PacketID};

fn main() {
    esp_idf_svc::sys::link_patches(); //Needed for esp32-rs
    println!("Hello espnow!");

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();
    wifi_driver.start().unwrap();
    // TODO: Set channel for the Wifi driver?

    let espnow_driver = EspNow::take().unwrap();

    let peer_exists = espnow_driver.peer_exists(BROADCAST).unwrap();
    if !peer_exists {
        let mut peer_info = PeerInfo::default();
        peer_info.peer_addr = BROADCAST;
        peer_info.channel = 0;
        peer_info.encrypt = false;

        espnow_driver.add_peer(peer_info).unwrap();
    }

    let mut packet_id: PacketID = 0;
    loop {
        let packet = Packet::new(packet_id, 'A', 'B');
        let serialized = bincode::serialize(&packet).unwrap();
        espnow_driver
            .send(BROADCAST, &serialized.as_slice())
            .unwrap();

        println!("Sent {:?} as broadcast!", packet);
        sleep(Duration::new(2, 0));
        packet_id += 1;
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Error;

    #[test_case]
    fn it_works() -> Result<(), Error> {
        let result = 2 + 2;

        anyhow::ensure!(result == 4);

        return Ok(());
    }

    #[test_case]
    fn it_doesnt_work() -> Result<(), Error> {
        let result = 2 + 6;

        anyhow::ensure!(result == 4, "result should be equal to {}", 4);

        return Ok(());
    }
}
