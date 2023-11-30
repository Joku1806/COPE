# COPE

This is an implementation of the [COPE protocol](https://dl.acm.org/doi/abs/10.1145/1159913.1159942) for the ESP32-S3 written in Rust.

## Installation

First, follow the rust toolchain [installation instructions](https://esp-rs.github.io/book/installation/index.html) of the esp-rs book. However, as rust on embedded devices is still quite unstable, do not install the latest available toolchain! We have had crates unable to build after a toolchain upgrade. To install our supported toolchain, run `espup install --nightly-version nightly-2023-10-05 --toolchain-version 1.73.0 --targets esp32s3 --std` instead of the command given in the esp-rs book.

## Directory Structure

The implementation is made up of three different directories:

1. `cope`, which contains an implementation of the COPE protocol and platform abstractions
2. `simulator`, which simulates the protocol on your host machine
3. `esp`, which runs the protocol on the ESP32

## Running

To run the simulator, enter the `simulator` directory and run `cargo run`. Likewise, to run on the ESP32, enter the `esp` directory and run `cargo run`.
