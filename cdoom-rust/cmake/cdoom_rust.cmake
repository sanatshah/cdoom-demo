# Builds cdoom-core as a static library and exposes it to Chocolate Doom CMake.

get_filename_component(CDOOM_RUST_DIR "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)

find_program(CARGO_EXECUTABLE cargo REQUIRED)

set(CDOOM_RUST_PROFILE "release" CACHE STRING "Cargo profile for cdoom-rust")
set(CDOOM_RUST_TARGET_DIR "${CDOOM_RUST_DIR}/target")
set(CDOOM_RUST_LIB "${CDOOM_RUST_TARGET_DIR}/${CDOOM_RUST_PROFILE}/libcdoom_core.a")
set(CDOOM_RUST_HEADER "${CDOOM_RUST_DIR}/include/cdoom_rust.h")

if(APPLE)
    set(CDOOM_RUST_LINK_LIBS "")
elseif(UNIX)
    set(CDOOM_RUST_LINK_LIBS m dl pthread)
elseif(WIN32)
    set(CDOOM_RUST_LINK_LIBS ws2_32 userenv bcrypt)
endif()

add_custom_command(
    OUTPUT "${CDOOM_RUST_LIB}" "${CDOOM_RUST_HEADER}"
    COMMAND ${CARGO_EXECUTABLE} build
            --manifest-path "${CDOOM_RUST_DIR}/Cargo.toml"
            --profile "${CDOOM_RUST_PROFILE}"
            -p cdoom-core
    WORKING_DIRECTORY "${CDOOM_RUST_DIR}"
    COMMENT "Building cdoom-core (${CDOOM_RUST_PROFILE})"
    VERBATIM
)

add_custom_target(cdoom_rust_build DEPENDS "${CDOOM_RUST_LIB}" "${CDOOM_RUST_HEADER}")

add_library(cdoom_rust STATIC IMPORTED GLOBAL)
add_dependencies(cdoom_rust cdoom_rust_build)
set_target_properties(cdoom_rust PROPERTIES
    IMPORTED_LOCATION "${CDOOM_RUST_LIB}"
    INTERFACE_INCLUDE_DIRECTORIES "${CDOOM_RUST_DIR}/include"
    INTERFACE_LINK_LIBRARIES "${CDOOM_RUST_LINK_LIBS}"
)

add_executable(cdoom_rust_probe "${CDOOM_RUST_DIR}/tools/probe.c")
target_link_libraries(cdoom_rust_probe PRIVATE cdoom_rust)
add_dependencies(cdoom_rust_probe cdoom_rust_build)

# Attach Rust headers/flags to game executables. Linking is done via EXTRA_LIBS.
function(cdoom_rust_link target)
    add_dependencies("${target}" cdoom_rust_build)
    target_include_directories("${target}" PRIVATE "${CDOOM_RUST_DIR}/include")
    target_compile_definitions("${target}" PRIVATE ENABLE_CDOOM_RUST=1)
endfunction()

message(STATUS "cdoom-rust: ${CDOOM_RUST_LIB}")
