cmake_minimum_required(VERSION 3.16)
include($ENV{IDF_PATH}/tools/cmake/project.cmake)

if(${IDF_TARGET} STREQUAL "linux")
    add_compile_options(-DCOPE_TEST_LINUX)
endif()

list(APPEND EXTRA_COMPONENT_DIRS "../components")
list(APPEND TEST_COMPONENTS "testable")
set(COMPONENTS testable main)

project(test_runner)
