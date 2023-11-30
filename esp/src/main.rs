#![feature(custom_test_frameworks)]
#![test_runner(test_runner::run)]

mod esp_channel;
mod test_runner;

use cope::traffic_generator::poisson_generator::PoissonGenerator;
use cope::Node;
use std::{thread::sleep, time::Duration};

use byte_unit::{Byte, Unit};

use crate::esp_channel::EspChannel;

fn main() {
    esp_idf_svc::sys::link_patches();
    println!("Hello espnow!");

    let mut channel = EspChannel::new();
    channel.initialize();
    let traffic_generator =
        PoissonGenerator::new(Byte::from_u64_with_unit(2, Unit::KB).unwrap().as_u64() as u32);
    let mut node = Node::new(
        'A',
        'B',
        Vec::from(['B']),
        Box::new(channel),
        Box::new(traffic_generator),
    );

    loop {
        node.tick();
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
