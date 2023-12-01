#!/bin/bash

logfile="test.log"
esp_port=$1

# Between different computers, the specific serial port
# will also most likely be different. Because of this,
# we support passing the serial port as a parameter.
if [ -n "$esp_port" ]; then
    cargo test -- -p "$esp_port" > "$logfile" 2>&1 &
else
    # If you did not set a default serial port using cargo,
    # espflash will ask about it and block script execution!
    cargo test > "$logfile" 2>&1 &
fi

pid=$!

while sleep 1
do
    if grep -F --quiet "Returned from app_main" "$logfile"; then
        # Needs to kill all child processes because cargo test
        # spawns espflash as a child process, which would otherwise
        # block the serial port on subsequent runs.
        pkill -TERM -P $pid
        break
    fi
done

# \K\d+ also contains the " failed" at the end for some reason.
# So cut is needed to just extract the number.
failures=$(grep -Po "Ran \d+ tests, \d+ passed and \K\d+ failed." test.log | cut -d ' ' -f 1)

cat $logfile
rm $logfile

# Don't know what exactly, but something in this script
# messes up the terminal output completely.
# To clean this up, reset to a good state.
reset

if (( failures > 0 )); then
    exit 1
fi
