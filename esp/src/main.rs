#![feature(custom_test_frameworks)]
#![test_runner(test_runner::run)]

mod esp_channel;
mod test_runner;

use std::time::Duration;

use cope::config::CONFIG;
use cope::Node;
use cope_config::types::mac_address::MacAddress;

use enumset::enum_set;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        cpu::Core,
        peripherals::Peripherals,
        task::watchdog::{TWDTConfig, TWDTDriver},
    },
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi, WifiDeviceId},
};

use crate::esp_channel::EspChannel;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    // TODO: Move all this init stuff to a separate function
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // TODO: Better error handling
    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();
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
    let mac = MacAddress::from(wifi_driver.get_mac(WifiDeviceId::Sta).unwrap());

    let watchdog_config = TWDTConfig {
        duration: Duration::from_secs(2),
        panic_on_trigger: true,
        // NOTE: Make sure that the IDLE task always runs on this core!
        // The watchdog example uses Core::Core0 instead.
        subscribed_idle_tasks: enum_set!(Core::Core1),
    };
    let mut driver = TWDTDriver::new(peripherals.twdt, &watchdog_config)?;
    let mut watchdog = driver.watch_current_task()?;

    let mut channel = EspChannel::new();
    channel.initialize();

    let id = CONFIG
        .get_node_id_for(mac)
        .expect("Config should contain Node MAC addresses");
    let mut node = Node::new(id, Box::new(channel));

    loop {
        node.tick();
        let _ = watchdog.feed();
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
