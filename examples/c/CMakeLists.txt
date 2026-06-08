cmake_minimum_required(VERSION 3.15)
project(wickra_c_examples C CXX)

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

enable_testing()

# Build one example, link it to the Wickra library, run it as a ctest. On Windows
# the DLL is copied next to the executable so the loader finds it at run time.
function(add_wickra_example name source)
    add_executable(${name} ${source})
    target_include_directories(${name} PRIVATE "${WICKRA_INCLUDE_DIR}")
    target_link_libraries(${name} PRIVATE "${WICKRA_LINK_LIB}")
    if(UNIX AND NOT APPLE)
        target_link_libraries(${name} PRIVATE m)
    endif()
    if(WIN32)
        add_custom_command(TARGET ${name} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
                    "${WICKRA_RUNTIME}" "$<TARGET_FILE_DIR:${name}>")
    endif()
    add_test(NAME ${name} COMMAND ${name})
    if(NOT WIN32)
        set_tests_properties(${name} PROPERTIES
            ENVIRONMENT "LD_LIBRARY_PATH=${WICKRA_LIB_DIR};DYLD_LIBRARY_PATH=${WICKRA_LIB_DIR}")
    endif()
endfunction()

add_wickra_example(smoke smoke.c)         # links the boundary, asserts values
add_wickra_example(streaming streaming.c) # runnable streaming demo
add_wickra_example(cpp_smoke smoke.cpp)   # C++ RAII wrapper (wickra.hpp)
