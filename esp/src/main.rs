#![feature(custom_test_frameworks)]
#![test_runner(test_runner::run)]

mod esp_channel;
mod espnow_frame;
mod frame_collection;
mod promiscuous_wifi;
mod test_runner;
mod wifi_frame;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use cope::config::CONFIG;
use cope::stats::{Stats, StatsLogger};
use cope::Node;
use esp_channel::EspStatsLogger;
use simple_logger::SimpleLogger;

use enumset::enum_set;
use esp_idf_svc::hal::{
    cpu::Core,
    peripherals::Peripherals,
    task::watchdog::{TWDTConfig, TWDTDriver},
};

use crate::esp_channel::EspChannel;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()?;

    let peripherals = Peripherals::take()?;
    let mut esp_channel = EspChannel::new(peripherals.modem)?;
    esp_channel.initialize()?;

    // TODO: Investigate, why we apparently don't reset the watchdog sometimes. To
    // mask this issue, I just set it to not panic for now.
    let watchdog_config = TWDTConfig {
        duration: Duration::from_secs(30),
        panic_on_trigger: false,
        // NOTE: Make sure that the IDLE task always runs on this core! The watchdog example uses
        // Core::Core0 instead.
        subscribed_idle_tasks: enum_set!(Core::Core1),
    };
    let mut driver = TWDTDriver::new(peripherals.twdt, &watchdog_config)?;
    let mut watchdog = driver.watch_current_task()?;

    let mac = esp_channel.get_mac();
    log::info!("Read MAC address (Sta): {}", mac);

    let id = CONFIG
        .get_node_id_for(mac)
        .expect("Config should contain Node MAC addresses");

    let logger = EspStatsLogger::new(format!("./log/esp/log_{}", id.unwrap()).as_str()).unwrap();
    let stats = Arc::new(Mutex::new(Stats::new(id, Box::new(logger))));
    esp_channel.set_statistics(&stats);
    let mut node = Node::new(id, Box::new(esp_channel), &stats);

    loop {
        node.tick();
        let _ = watchdog.feed();
    }
}
