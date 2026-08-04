---
name: ffi-bridge
description: Wires cdoom-core Rust modules into Chocolate Doom via ffi.rs, cbindgen, and C dual-run shims. Use when exporting Rust functions, adding cdoom_rust_* symbols, or integrating USE_RUST_* in C callers.
---

# cdoom FFI Bridge

Connects Rust implementations to the vendored C game through a stable C ABI.

## Architecture

```
cdoom-core/src/<module>.rs   ← logic
        ↓
cdoom-core/src/ffi.rs        ← #[no_mangle] extern "C" wrappers
        ↓ (build.rs + cbindgen)
cdoom-rust/include/cdoom_rust.h
        ↓
chocolate-doom/src/*.c         ← #ifdef USE_RUST_* dual-run callers
```

CMake (`cdoom-rust/cmake/cdoom_rust.cmake`) builds `libcdoom_core.a` and exposes headers via `cdoom_rust_link()`.

## Adding an export

### 1. Implement in a module

```rust
// cdoom-core/src/m_fixed.rs
pub fn fixed_mul(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> 16) as i32
}
```

### 2. Thin wrapper in ffi.rs

```rust
use std::os::raw::c_int;

/// # Safety
/// Only called from C with valid fixed-point values.
#[no_mangle]
pub extern "C" fn cdoom_rust_fixed_mul(a: c_int, b: c_int) -> c_int {
    crate::m_fixed::fixed_mul(a, b)
}
```

### 3. Rebuild to regenerate header

```bash
cargo build --manifest-path cdoom-rust/Cargo.toml -p cdoom-core
# writes cdoom-rust/include/cdoom_rust.h
```

### 4. Wire C caller (dual-run)

```c
#ifdef USE_RUST_M_FIXED
#include "cdoom_rust.h"
#endif

fixed_t FixedMul(fixed_t a, fixed_t b)
{
#ifdef USE_RUST_M_FIXED
    return cdoom_rust_fixed_mul(a, b);
#else
    return ((int64_t)a * (int64_t)b) >> FRACBITS;
#endif
}
```

## Naming rules (cbindgen.toml)

- All exports prefixed `cdoom_rust_` via `[export] prefix`.
- Use `std::os::raw` types at the boundary: `c_int`, `c_char`, `c_void`, not Rust `usize`.
- `usize_is_size_t = true` — size parameters use platform `size_t`.

## Pointer and memory rules

| Rule | Reason |
|---|---|
| Return `*const c_char` only to `'static` data (`OnceLock<CString>`, string literals) | C must not free |
| Never return stack pointers | Immediate use-after-free |
| Document `# Safety` on every `unsafe` block and raw-pointer function | FFI contract |
| If C must free, provide paired `cdoom_rust_*_free` and document ownership | Match C conventions |

```rust
// ✅ static lifetime
static VERSION: OnceLock<CString> = OnceLock::new();

#[no_mangle]
pub extern "C" fn cdoom_rust_version() -> *const c_char {
    VERSION.get_or_init(|| CString::new(env!("CARGO_PKG_VERSION")).unwrap()).as_ptr()
}

// ❌ dangling pointer
#[no_mangle]
pub extern "C" fn cdoom_rust_bad() -> *const c_char {
    CString::new("tmp").unwrap().as_ptr()  // freed at return
}
```

## Out-parameters and buffers

When C passes writable buffers:

```rust
#[no_mangle]
pub extern "C" fn cdoom_rust_fill_digest(out: *mut u8, out_len: usize) -> c_int {
    if out.is_null() || out_len < 20 {
        return -1;
    }
    let digest = compute_digest();
    unsafe {
        std::ptr::copy_nonoverlapping(digest.as_ptr(), out, 20);
    }
    0
}
```

Document required buffer sizes in the `# Safety` doc comment.

## Opaque handles

For stateful C objects (file handles, hash contexts):

```rust
pub struct Sha1Context { /* ... */ }

#[no_mangle]
pub extern "C" fn cdoom_rust_sha1_init(ctx: *mut Sha1Context) {
    assert!(!ctx.is_null());
    unsafe { ctx.write(Sha1Context::new()) };
}
```

Or use `Box::into_raw` / `Box::from_raw` for heap-allocated opaque pointers when C expects a pointer token.

## CMake integration

Production linking is gated by `ENABLE_CDOOM_RUST` (see `chocolate-doom/CMakeLists.txt`). Per-module routing uses compile definitions:

```cmake
# Add when module parity is proven:
target_compile_definitions(${target} PRIVATE USE_RUST_M_FIXED=1)
```

Add module flags only after parity tests pass — never before.

## Quick validation

```bash
# Regenerate header + build staticlib
cargo build --manifest-path cdoom-rust/Cargo.toml -p cdoom-core

# Standalone FFI smoke test (needs prior ./build.sh)
./chocolate-doom/build/cdoom_rust_probe

# In-game version check
./chocolate-doom/build/src/chocolate-doom -cdoom-rust-info
```

## Do not

- Edit `include/cdoom_rust.h` by hand.
- Put business logic in `ffi.rs` — wrappers only.
- Use Rust panics across the FFI boundary (abort the process or return error codes).
- Export `pub` Rust APIs without `#[no_mangle] extern "C"` and expect C to link them.
