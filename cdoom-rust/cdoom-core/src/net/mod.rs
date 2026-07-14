//! Networking helpers migrated from Chocolate Doom's shared `net_*` code.
//!
//! This module intentionally mirrors the C wire format rather than inventing
//! new Rust-level packet abstractions. C still owns packet allocation and I/O.

pub const NET_PROTOCOL_CHOCOLATE_DOOM_0: i32 = 0;
pub const NET_NUM_PROTOCOLS: i32 = 1;
pub const NET_PROTOCOL_UNKNOWN: i32 = 2;

const PROTOCOL_CHOCOLATE_DOOM_0: &[u8] = b"CHOCOLATE_DOOM_0";

/// Expand the least significant tic byte around a relative tic number.
pub fn expand_tic_num(relative: u32, b: u32) -> u32 {
    let high = relative & !0xff;
    let low = relative & 0xff;
    let mut result = high | b;

    if low < 0x40 && b > 0xb0 {
        result = result.wrapping_sub(0x100);
    }
    if low > 0xb0 && b < 0x40 {
        result = result.wrapping_add(0x100);
    }

    result
}

pub fn read_u8(data: &[u8], pos: &mut u32) -> Option<u32> {
    let start = usize::try_from(*pos).ok()?;
    let end = start.checked_add(1)?;
    if end > data.len() {
        return None;
    }

    *pos = end as u32;
    Some(data[start] as u32)
}

pub fn read_u16(data: &[u8], pos: &mut u32) -> Option<u32> {
    let start = usize::try_from(*pos).ok()?;
    let end = start.checked_add(2)?;
    if end > data.len() {
        return None;
    }

    *pos = end as u32;
    Some(u16::from_be_bytes([data[start], data[start + 1]]) as u32)
}

pub fn read_u32(data: &[u8], pos: &mut u32) -> Option<u32> {
    let start = usize::try_from(*pos).ok()?;
    let end = start.checked_add(4)?;
    if end > data.len() {
        return None;
    }

    *pos = end as u32;
    Some(u32::from_be_bytes([
        data[start],
        data[start + 1],
        data[start + 2],
        data[start + 3],
    ]))
}

pub fn read_i8(data: &[u8], pos: &mut u32) -> Option<i32> {
    read_u8(data, pos).map(|value| value as i8 as i32)
}

pub fn read_i16(data: &[u8], pos: &mut u32) -> Option<i32> {
    read_u16(data, pos).map(|value| value as i16 as i32)
}

pub fn read_i32(data: &[u8], pos: &mut u32) -> Option<i32> {
    read_u32(data, pos).map(|value| value as i32)
}

pub fn write_u8(data: &mut [u8], value: u32) -> bool {
    if data.is_empty() {
        return false;
    }

    data[0] = value as u8;
    true
}

pub fn write_u16(data: &mut [u8], value: u32) -> bool {
    if data.len() < 2 {
        return false;
    }

    data[..2].copy_from_slice(&(value as u16).to_be_bytes());
    true
}

pub fn write_u32(data: &mut [u8], value: u32) -> bool {
    if data.len() < 4 {
        return false;
    }

    data[..4].copy_from_slice(&value.to_be_bytes());
    true
}

pub fn parse_protocol_name(name: &[u8]) -> i32 {
    if name == PROTOCOL_CHOCOLATE_DOOM_0 {
        NET_PROTOCOL_CHOCOLATE_DOOM_0
    } else {
        NET_PROTOCOL_UNKNOWN
    }
}

pub fn protocol_name(protocol: i32) -> Option<&'static [u8]> {
    match protocol {
        NET_PROTOCOL_CHOCOLATE_DOOM_0 => Some(PROTOCOL_CHOCOLATE_DOOM_0),
        _ => None,
    }
}

pub fn write_protocol_list() -> Vec<u8> {
    let mut result = Vec::new();
    result.push(NET_NUM_PROTOCOLS as u8);
    result.extend_from_slice(PROTOCOL_CHOCOLATE_DOOM_0);
    result.push(0);
    result
}
