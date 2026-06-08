cmake_minimum_required(VERSION 3.15)
project(wickra_c_examples C)

# Directory holding the compiled Wickra C library (cargo output), e.g.
# <workspace>/target/release. Override with -DWICKRA_LIB_DIR=/path/to/target/release.
if(NOT DEFINED WICKRA_LIB_DIR)
    set(WICKRA_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release")
endif()
set(WICKRA_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../bindings/c/include")

# Pick the right link target per platform/toolchain.
#  - MSVC links the generated import library (wickra.dll.lib).
#  - MinGW/gcc on Windows links the DLL directly.
#  - Unix links the shared object / dylib.
if(WIN32)
    set(WICKRA_RUNTIME "${WICKRA_LIB_DIR}/wickra.dll")
    if(MSVC)
        set(WICKRA_LINK_LIB "${WICKRA_LIB_DIR}/wickra.dll.lib")
    else()
        set(WICKRA_LINK_LIB "${WICKRA_LIB_DIR}/wickra.dll")
    endif()
elseif(APPLE)
    set(WICKRA_LINK_LIB "${WICKRA_LIB_DIR}/libwickra.dylib")
else()
    set(WICKRA_LINK_LIB "${WICKRA_LIB_DIR}/libwickra.so")
endif()

add_executable(smoke smoke.c)
target_include_directories(smoke PRIVATE "${WICKRA_INCLUDE_DIR}")
target_link_libraries(smoke PRIVATE "${WICKRA_LINK_LIB}")
if(UNIX AND NOT APPLE)
    target_link_libraries(smoke PRIVATE m)
endif()

# On Windows the DLL must sit next to the executable at run time.
if(WIN32)
    add_custom_command(TARGET smoke POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${WICKRA_RUNTIME}" "$<TARGET_FILE_DIR:smoke>")
endif()

enable_testing()
add_test(NAME smoke COMMAND smoke)
# Unix needs the shared library on the loader path when the test runs.
if(NOT WIN32)
    set_tests_properties(smoke PROPERTIES
        ENVIRONMENT "LD_LIBRARY_PATH=${WICKRA_LIB_DIR};DYLD_LIBRARY_PATH=${WICKRA_LIB_DIR}")
endif()
