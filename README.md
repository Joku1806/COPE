# COPE

This is the repository for COPE, a protocol utilising opportunistic network coding to improve throughput in wireless (mesh) networks. This implementation is written in C++ and targets the ESP32 chipset.

## Building

To build this project, you need to install [esp-idf](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/). I don't actually know how to build from the command line because I just used the VSCode extension :P

## Editor Support

### VSCode

`esp-idf` can also be installed as a VSCode [extension](https://marketplace.visualstudio.com/items?itemName=espressif.esp-idf-extension). When building the project, `esp-idf` generates `build/compile_commands.json`, which the `C/C++ Extension` can use to provide autocomplete, go to definition and other nice things. To enable this, run `C/C++: Edit Configurations (UI)` from the command palette (`Ctrl+Shift+P`). Under the advanced section, set the compile commands path to `${workspaceFolder}/build/compile_commands.json`. You will also need to change the compiler path to the `esp-idf` compiler. Otherwise, wrong standard library includes will be found. To find the location of the esp compiler, run `find /path/to/.espressif -name '*esp32s3*gcc' -executable`. If there are still errors after changing the compiler path, you may need to reload the window. After that, IntelliSense should work without problems.