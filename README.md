# COPE

This is an implementation of the [COPE protocol](https://dl.acm.org/doi/abs/10.1145/1159913.1159942) for the ESP32-S3 written in Rust.

## Installation

To install all required dependencies for running all of our code, including the simulator and on the ESP, as well as our plot scripts, execute `install.sh` in the project root directory. We assume that you use a normal linux distribution, a recent `bash` installation and have `sudo` permissions.

## Directory Structure

The implementation is made up of three different directories:

1. `cope`, which contains an implementation of the COPE protocol and platform abstractions
2. `simulator`, which simulates the protocol on your host machine
3. `esp`, which runs the protocol on the ESP32

## Running

To run the simulator, enter the `simulator` directory and run `cargo run`. Likewise, to run on the ESP32, enter the `esp` directory and run `cargo run`. Statistics can be collected on the ESP by running `Meta/collect_statistics.sh` instead.

By default, all logging is disabled. It can be enabled again by changing the global log level for `SimpleLogger` in the respective `main.rs` files.

To run our plot scripts, source the python `venv` created by `install.sh` and then run `python main.py ../logs/raw_throughput_1Mbit` from the `plot_script` directory. Not all data can be plotted using all plots, if you run into any errors, just comment out the offending plots.

## Debugging

For a better debugging experience, install the "time-travelling" debugger [rr](https://rr-project.org/). Inside the `simulator` subdirectory, there is a custom `.gdbinit` file, which is needed for `rr` to print rust variables. To be able to load this file, you need to add the line `set auto-load safe-path .` to your global `~/.gdbinit` file. After that, you can record a simulator run using `rr record target/debug/simulator` and replay it using `rr replay`.

Heap memory usage can be analyzed for example using `heaptrack` on the simulator, but many other tools should work as well.
