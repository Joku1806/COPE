#![feature(custom_test_frameworks)]
#![test_runner(test_runner::run)]

mod esp_channel;
mod esp_stats_logger;
mod espnow_frame;
mod espnow_stats;
mod frame_collection;
mod promiscuous_wifi;
mod test_runner;
mod wifi_frame;

use std::time::Duration;

use cope::stats::Stats;
use cope::Node;
use cope::{config::CONFIG, stats::StatsLogger};
use simple_logger::SimpleLogger;

use enumset::enum_set;
use esp_idf_svc::hal::{
    cpu::Core,
    peripherals::Peripherals,
    task::watchdog::{TWDTConfig, TWDTDriver},
};

use crate::{esp_channel::EspChannel, esp_stats_logger::EspStatsLogger};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
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
    let stats = Stats::new(id, Box::new(logger), CONFIG.stats_log_duration);
    let mut node = Node::new(id, Box::new(esp_channel), stats);

    loop {
        node.tick();
        let _ = watchdog.feed();
    }
}
