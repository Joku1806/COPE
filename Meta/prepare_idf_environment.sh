#!/bin/bash

if [[ ! -v IDF_PATH ]]; then
    echo "Please set the IDF_PATH environment variable to your esp-idf directory!"
    exit 1
fi

source $IDF_PATH/export.sh > /dev/null || (
    echo "IDF_PATH does not point to an esp-idf installation!"
    exit 1
)
