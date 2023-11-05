#!/bin/bash

esp32_port="/dev/ttyACM0"

print_usage_and_exit() {
    echo "Usage: Meta/run_tests.sh [ linux | esp32s3 ]"
    exit 1
}

run_tests_linux() {
    idf.py --preview set-target linux > /dev/null
    idf.py clean build > /dev/null

    test_output=$(./build/test_runner.elf)
    echo "$test_output"
    n_failures=$(echo "$test_output" | grep -c ":FAIL")

    if (( n_failures > 0 )); then
        exit 1
    fi
}

run_tests_esp32s3() {
    idf.py --preview set-target esp32s3 > /dev/null
    # TODO: Find a way to get test output without monitor
    idf.py -p ${esp32_port} clean flash monitor > /dev/null
}

source Meta/prepare_idf_environment.sh || print_usage_and_exit
cd test || print_usage_and_exit
target=$1

if [ "$target" == "esp32s3" ]; then
    run_tests_esp32s3
elif [ "$target" == "linux" ]; then
    run_tests_linux
else
    print_usage_and_exit
fi
