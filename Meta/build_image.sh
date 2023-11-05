#!/bin/bash

print_usage_and_exit() {
    echo "Usage: Meta/build_image.sh"
    exit 1
}

source Meta/prepare_idf_environment.sh || print_usage_and_exit

idf.py --preview set-target esp32s3 > /dev/null
idf.py clean build > /dev/null
