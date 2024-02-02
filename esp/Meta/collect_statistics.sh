#!/bin/bash

esp_port=$1

process_output() {
    while read -r line; do
        if [[ $line == STATS* ]]; then
            mapfile -d " " -t parts < <(printf "%s" "$line")
            # NOTE: For the format of statistics output,
            # see the implementation of EspStatsLogger in esp_channel.rs
            filename="${parts[1]}"
            data="${parts[2]}"
            mkdir -p "${filename%/*}"
            echo "${data}" >> "${filename}"
        else
            echo "${line}"
        fi
    done
}

# Between different computers, the specific serial port
# will also most likely be different. Because of this,
# we support passing the serial port as a parameter.
if [ -n "$esp_port" ]; then
    cargo run -- -p "$esp_port" | process_output
else
    # If you did not set a default serial port using cargo,
    # espflash will ask about it and block script execution!
    cargo run | process_output
fi
