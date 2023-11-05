/* Example test application for testable component.

   This example code is in the Public Domain (or CC0 licensed, at your option.)

   Unless required by applicable law or agreed to in writing, this
   software is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
   CONDITIONS OF ANY KIND, either express or implied.
*/

#include "unity.h"
#include <stdio.h>
#include <string.h>

void app_main(void)
{
    UNITY_BEGIN();
    unity_run_all_tests();
    UNITY_END();

#if defined(COPE_TEST_LINUX)
    // NOTE: On linux, we would hang indefinitely, because the main task is not
    // stopped. This makes sense on a microcontroller, since the task would
    // immediately restart after killing it. But on linux, this is not the case.
    // We also don't want to wait until CI kills the test task, so we should exit
    // ourselves, if the test code runs on linux.
    //
    // esp-idf does not set a platform define itself, so we inject our own define
    // in CMakeLists.txt one directory up.
    exit(0);
#endif
}
