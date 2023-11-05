# TODO: Get this running on an Alpine Image, ubuntu takes too much disk space
FROM ubuntu:23.10

RUN apt update
RUN apt install -y git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libbsd-dev libffi-dev libssl-dev dfu-util libusb-1.0-0

RUN adduser --system --group --home /home/idftester idftester
USER idftester

WORKDIR /home/idftester
RUN git clone -b release/v5.1 --recursive https://github.com/espressif/esp-idf.git

WORKDIR /home/idftester/esp-idf
RUN ./install.sh esp32s3

WORKDIR /home/idftester
RUN printf "shopt -s expand_aliases\nalias get_idf='. $HOME/esp-idf/export.sh'" >> .profile

CMD /bin/bash
