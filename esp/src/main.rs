#![feature(custom_test_frameworks)]
#![test_runner(test_runner::run)]

mod esp_channel;
mod test_runner;

use cope::Node;
use std::{thread::sleep, time::Duration};

use crate::esp_channel::EspChannel;

fn main() {
    esp_idf_svc::sys::link_patches();
    println!("Hello espnow!");

    let mut channel = EspChannel::new();
    channel.initialize();
    let mut node = Node::new('A', 'B', Vec::from(['B']), Box::new(channel));

    let mut packet_id: PacketID = 0;
    loop {
        broadcast_counter += 1;
        node.tick();
        println!("Sent broadcast message no. {}!", broadcast_counter);
        sleep(Duration::new(1, 0));
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
