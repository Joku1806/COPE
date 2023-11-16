# Practical Network Coding with ESP32

This is the repository for `COPE`, a protocol utilising opportunistic network coding to improve throughput in wireless (mesh) networks. This implementation is written in Rust and targets the ESP32 chipset.

## Building

To build this project, follow the installation instruction for the ESP32 rust toolchain [here](https://esp-rs.github.io/book/introduction.html). You will only need the `std` toolchain. Then to build the project, just execute `cargo build` and everything should hopefully work.

## Editor Support

### VSCode

VSCode has the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension that works out of the box. If you run into any problems, check if anything [here](https://esp-rs.github.io/book/tooling/visual-studio-code.html) fixes it.
