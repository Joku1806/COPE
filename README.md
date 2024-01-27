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

### Running on the lab machines

The lab machines do not have the `libuv-dev` dependency available that is needed to build espflash. Instead of the espflash install command given in the esp-rs book, run `Meta/install_lab_dependencies.sh` while inside the `esp` directory, which installs a prebuilt version of espflash to `~/.local/bin`.

The newest version of espflash that works on the lab machines is v1.7.0. Because of this, you will need to set the `runner` variable in `.cargo/config.toml` to `espflash --monitor` instead of the current command for `2.x.x` espflash.

## Debugging

For a better debugging experience, install the "time-travelling" debugger [rr](https://rr-project.org/). Inside the `simulator` subdirectory, there is a custom `.gdbinit` file, which is needed for `rr` to print rust variables. To be able to load this file, you need to add the line `set auto-load safe-path .` to your global `~/.gdbinit` file. After that, you can record a simulator run using `rr record target/debug/simulator` and replay it using `rr replay`.
