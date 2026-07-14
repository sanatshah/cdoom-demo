//! Stable C ABI exported to Chocolate Doom via `cdoom_rust.h`.

use std::ffi::{c_void, CString};
use std::os::raw::{c_char, c_int, c_long, c_uint};
use std::sync::OnceLock;

use crate::{aes_prng, memio, sha1};

static VERSION: OnceLock<CString> = OnceLock::new();

fn version_cstr() -> &'static CString {
    VERSION.get_or_init(|| {
        CString::new(env!("CARGO_PKG_VERSION")).expect("version must not contain NUL")
    })
}

/// Returns a pointer to a static, NUL-terminated version string.
///
/// # Safety
///
/// The returned pointer is valid for the process lifetime and must not be freed.
#[no_mangle]
pub extern "C" fn cdoom_rust_version() -> *const c_char {
    version_cstr().as_ptr()
}

/// One-time initialization hook for future Rust subsystems.
///
/// Returns `0` on success. Reserved for later migration phases.
#[no_mangle]
pub extern "C" fn cdoom_rust_init() -> i32 {
    0
}

/// Opens an existing buffer for `memio` reads.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_mem_fopen_read(buf: *mut c_void, buflen: usize) -> *mut c_void {
    memio::fopen_read(buf, buflen)
}

/// Opens a Rust-owned `memio` write buffer.
#[no_mangle]
pub extern "C" fn cdoom_rust_mem_fopen_write() -> *mut c_void {
    memio::fopen_write()
}

/// Reads from a Rust `memio` stream.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_mem_fread(
    buf: *mut c_void,
    size: usize,
    nmemb: usize,
    stream: *mut c_void,
) -> usize {
    memio::fread(buf, size, nmemb, stream)
}

/// Writes to a Rust `memio` stream.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_mem_fwrite(
    ptr: *const c_void,
    size: usize,
    nmemb: usize,
    stream: *mut c_void,
) -> usize {
    memio::fwrite(ptr, size, nmemb, stream)
}

/// Returns the current Rust `memio` backing buffer and length.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_mem_get_buf(
    stream: *mut c_void,
    buf: *mut *mut c_void,
    buflen: *mut usize,
) {
    memio::get_buf(stream, buf, buflen);
}

/// Closes a Rust `memio` stream.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_mem_fclose(stream: *mut c_void) {
    memio::fclose(stream);
}

/// Returns the Rust `memio` stream position.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_mem_ftell(stream: *mut c_void) -> c_long {
    memio::ftell(stream) as c_long
}

/// Seeks within a Rust `memio` stream.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_mem_fseek(
    stream: *mut c_void,
    offset: c_long,
    whence: c_int,
) -> c_int {
    memio::fseek(stream, offset as i64, whence)
}

/// Initializes a SHA-1 context using Rust.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_sha1_init(context: *mut c_void) {
    sha1::init(context);
}

/// Updates a SHA-1 context using Rust.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_sha1_update(
    context: *mut c_void,
    buf: *const c_void,
    len: usize,
) {
    sha1::update(context, buf, len);
}

/// Finalizes a SHA-1 digest using Rust.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_sha1_final(digest: *mut c_void, context: *mut c_void) {
    sha1::final_digest(digest, context);
}

/// Updates SHA-1 with a big-endian 32-bit integer using Rust.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_sha1_update_int32(context: *mut c_void, value: c_uint) {
    sha1::update_int32(context, value);
}

/// Updates SHA-1 with a NUL-terminated C string, including the NUL byte.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_sha1_update_string(context: *mut c_void, str: *const c_char) {
    sha1::update_string(context, str);
}

/// Starts the AES-PRNG from a 16-byte seed.
#[no_mangle]
pub unsafe extern "C" fn cdoom_rust_prng_start(seed: *const c_void) {
    aes_prng::start(seed);
}

/// Stops the AES-PRNG.
#[no_mangle]
pub extern "C" fn cdoom_rust_prng_stop() {
    aes_prng::stop();
}

/// Returns the next AES-PRNG value or zero when disabled.
#[no_mangle]
pub extern "C" fn cdoom_rust_prng_random() -> c_uint {
    aes_prng::random()
}
