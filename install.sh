#!/bin/bash

OWN_DIR=$(dirname -- "$0";)

# NOTE: Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# NOTE: Install the espup tool for easier setup for ESP32 toolchain
cargo install espup

# NOTE: Install our supported esp toolchain
espup install --nightly-version nightly-2023-10-05 --toolchain-version 1.73.0 --targets esp32s3 --std

# NOTE: Inform user about weird env variables that have to be sourced
echo "[*] espup has set up some env variables in $HOME/export-esp.sh:\n"
cat $HOME/export-esp.sh
echo "\n[*] Please make sure that these variables are present in every opened shell."
echo "[*] Otherwise the project will not build!"
echo "[*] For more information see https://esp-rs.github.io/book/installation/riscv-and-xtensa.html#3-set-up-the-environment-variables"

# NOTE: Install esp-idf dependencies
echo "[*] Installing esp-idf dependencies"
# NOTE: You need to be able to install stuff without sudo, I hope this works
apt-get install git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0

# NOTE: Install ldproxy, which is required for linking
echo "[*] Installing ldproxy"
cd $OWN_DIR/esp
cargo install ldproxy

# NOTE: Install a precompiled version of espflash, which works on the lab machines
cd /tmp || (
  echo "[!] User does not have a /tmp directory (How?)"
  exit 1
)

echo "[*] Installing espflash"
wget -q https://github.com/esp-rs/espflash/releases/download/v1.7.0/espflash-x86_64-unknown-linux-gnu.zip
unzip -q espflash-x86_64-unknown-linux-gnu.zip
chmod +x espflash
mv espflash ~/.local/bin/
rm espflash-x86_64-unknown-linux-gnu.zip
echo "[*] Installed espflash v1.7.0"

# NOTE: Install a python venv for running the plot scripts
cd /tmp || (
  echo "[!] User does not have a /tmp directory (How?)"
  exit 1
)

echo "[*] Installing python3 venv for plot scripts"
python3 -m venv COPE
echo "[*] Installed venv in /tmp/COPE, make sure to source it before running any plot scripts"

echo "[*] Installing plot script dependencies"
source /tmp/COPE/bin/activate
cd $OWN_DIR/plot_script
pip install -r requirements.txt

echo "[*] Complete installation finished!"
