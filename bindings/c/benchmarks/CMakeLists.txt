cmake_minimum_required(VERSION 3.15)
project(wickra_c_benchmarks C)

# Directory holding the compiled Wickra C library (cargo output), e.g.
# <workspace>/target/release. Override with -DWICKRA_LIB_DIR=/path/to/target/release.
if(NOT DEFINED WICKRA_LIB_DIR)
    set(WICKRA_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../target/release")
endif()
set(WICKRA_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../include")

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

add_executable(throughput throughput.c)
target_include_directories(throughput PRIVATE "${WICKRA_INCLUDE_DIR}")
target_link_libraries(throughput PRIVATE "${WICKRA_LINK_LIB}")
if(UNIX AND NOT APPLE)
    target_link_libraries(throughput PRIVATE m)
endif()

# On Windows copy the DLL next to the executable so the loader finds it.
if(WIN32)
    add_custom_command(TARGET throughput POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
                "${WICKRA_RUNTIME}" "$<TARGET_FILE_DIR:throughput>")
endif()
