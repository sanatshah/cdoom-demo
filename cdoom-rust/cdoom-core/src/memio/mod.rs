//! In-memory stdio emulation matching `memio.c`.

use std::ffi::c_void;
use std::ptr;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MemFileMode {
    Read,
    Write,
}

struct MemFile {
    buf: *mut u8,
    buflen: usize,
    alloced: usize,
    position: u32,
    mode: MemFileMode,
    owned: Option<Vec<u8>>,
}

impl MemFile {
    fn position(&self) -> usize {
        self.position as usize
    }
}

/// Opens an existing memory buffer for reading.
///
/// # Safety
///
/// `buf` must remain valid for `buflen` bytes until the returned stream is
/// closed.
pub unsafe fn fopen_read(buf: *mut c_void, buflen: usize) -> *mut c_void {
    let file = MemFile {
        buf: buf.cast(),
        buflen,
        alloced: 0,
        position: 0,
        mode: MemFileMode::Read,
        owned: None,
    };

    Box::into_raw(Box::new(file)).cast()
}

/// Opens a Rust-owned memory buffer for writing.
pub fn fopen_write() -> *mut c_void {
    let mut owned = Vec::with_capacity(1024);
    let file = MemFile {
        buf: owned.as_mut_ptr(),
        buflen: 0,
        alloced: 1024,
        position: 0,
        mode: MemFileMode::Write,
        owned: Some(owned),
    };

    Box::into_raw(Box::new(file)).cast()
}

/// Reads `nmemb` items of `size` bytes from `stream` into `buf`.
///
/// # Safety
///
/// `stream` must be a stream returned by this module and `buf` must be valid
/// for the number of bytes actually read.
pub unsafe fn fread(buf: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize {
    let Some(file) = stream.cast::<MemFile>().as_mut() else {
        return usize::MAX;
    };

    if file.mode != MemFileMode::Read {
        return usize::MAX;
    }

    let mut items = nmemb;
    let bytes_requested = items.wrapping_mul(size);
    let remaining = file.buflen.saturating_sub(file.position());

    if bytes_requested > remaining {
        items = remaining / size;
    }

    let bytes = items.wrapping_mul(size);
    if bytes != 0 {
        ptr::copy_nonoverlapping(file.buf.add(file.position()), buf.cast(), bytes);
    }
    file.position = file.position.wrapping_add(bytes as u32);

    items
}

/// Writes `nmemb` items of `size` bytes from `ptr` into `stream`.
///
/// # Safety
///
/// `stream` must be a stream returned by this module and `ptr` must be valid
/// for `size * nmemb` bytes.
pub unsafe fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize {
    let Some(file) = stream.cast::<MemFile>().as_mut() else {
        return usize::MAX;
    };

    if file.mode != MemFileMode::Write {
        return usize::MAX;
    }

    let bytes = size.wrapping_mul(nmemb);
    let pos = file.position();
    let required = pos.wrapping_add(bytes);

    if let Some(owned) = file.owned.as_mut() {
        while bytes > file.alloced.saturating_sub(pos) {
            file.alloced = file.alloced.saturating_mul(2);
            owned.reserve_exact(file.alloced.saturating_sub(owned.capacity()));
        }

        if required > owned.len() {
            owned.resize(required, 0);
        }
        if bytes != 0 {
            ptr::copy_nonoverlapping(ptr.cast::<u8>(), owned.as_mut_ptr().add(pos), bytes);
        }

        file.buf = owned.as_mut_ptr();
        file.buflen = file.buflen.max(required);
        file.position = file.position.wrapping_add(bytes as u32);
    }

    nmemb
}

/// Returns the backing buffer pointer and current logical length.
///
/// # Safety
///
/// `stream`, `buf`, and `buflen` must be valid pointers.
pub unsafe fn get_buf(stream: *mut c_void, buf: *mut *mut c_void, buflen: *mut usize) {
    let Some(file) = stream.cast::<MemFile>().as_mut() else {
        return;
    };

    if !buf.is_null() {
        *buf = file.buf.cast();
    }
    if !buflen.is_null() {
        *buflen = file.buflen;
    }
}

/// Closes a stream returned by this module.
///
/// # Safety
///
/// `stream` must be null or a stream returned by this module and not already
/// closed.
pub unsafe fn fclose(stream: *mut c_void) {
    if !stream.is_null() {
        drop(Box::from_raw(stream.cast::<MemFile>()));
    }
}

/// Returns the stream position.
///
/// # Safety
///
/// `stream` must be a stream returned by this module.
pub unsafe fn ftell(stream: *mut c_void) -> i64 {
    stream
        .cast::<MemFile>()
        .as_ref()
        .map_or(0, |file| i64::from(file.position))
}

/// Seeks within the stream using the `mem_rel_t` discriminants from C.
///
/// # Safety
///
/// `stream` must be a stream returned by this module.
pub unsafe fn fseek(stream: *mut c_void, position: i64, whence: i32) -> i32 {
    let Some(file) = stream.cast::<MemFile>().as_mut() else {
        return -1;
    };

    let newpos = match whence {
        0 => (position as i32) as u32,
        1 => ((i64::from(file.position)).wrapping_add(position) as i32) as u32,
        2 => (file.buflen as u64).wrapping_add(position as u64) as u32,
        _ => return -1,
    };

    if (newpos as usize) < file.buflen {
        file.position = newpos;
        0
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_stream_matches_c_edges() {
        let mut input = *b"abcdef";
        let stream = unsafe { fopen_read(input.as_mut_ptr().cast(), input.len()) };
        let mut out = [0_u8; 8];

        assert_eq!(unsafe { fread(out.as_mut_ptr().cast(), 2, 4, stream) }, 3);
        assert_eq!(&out[..6], b"abcdef");
        assert_eq!(unsafe { ftell(stream) }, 6);
        assert_eq!(unsafe { fseek(stream, 6, 0) }, -1);
        assert_eq!(unsafe { fseek(stream, -1, 2) }, 0);
        assert_eq!(unsafe { ftell(stream) }, 5);
        assert_eq!(unsafe { fread(out.as_mut_ptr().cast(), 0, 9, stream) }, 9);
        assert_eq!(unsafe { ftell(stream) }, 5);

        unsafe { fclose(stream) };
    }

    #[test]
    fn write_stream_grows_and_exposes_buffer() {
        let stream = fopen_write();
        let input = vec![0x5a; 1500];
        assert_eq!(
            unsafe { fwrite(input.as_ptr().cast(), 1, input.len(), stream) },
            input.len()
        );

        let mut buf = ptr::null_mut();
        let mut buflen = 0;
        unsafe { get_buf(stream, &mut buf, &mut buflen) };
        assert_eq!(buflen, input.len());
        let output = unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), buflen) };
        assert_eq!(output, input.as_slice());

        unsafe { fclose(stream) };
    }
}
