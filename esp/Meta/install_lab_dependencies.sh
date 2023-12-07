#!/bin/bash

cd /tmp || (
  echo "User does not have a /tmp directory (How?)"
  exit 1
)
wget -q https://github.com/esp-rs/espflash/releases/download/v1.7.0/espflash-x86_64-unknown-linux-gnu.zip
unzip -q espflash-x86_64-unknown-linux-gnu.zip
chmod +x espflash
mv espflash ~/.local/bin/
rm espflash-x86_64-unknown-linux-gnu.zip

echo "Installed espflash v1.7.0. Do not forget to change the runner in .cargo/config.toml!"
