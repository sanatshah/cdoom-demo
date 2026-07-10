//! Basic C type and keycode foundations mirrored from Chocolate Doom headers.

pub type Boolean = i32;
pub type Byte = u8;
pub type PixelT = u8;
pub type DPixelT = i16;
pub type FixedT = i32;
pub type AngleT = u32;

pub const FALSE: Boolean = 0;
pub const TRUE: Boolean = 1;

pub const FRACBITS: u32 = 16;
pub const FRACUNIT: FixedT = 1 << FRACBITS;

pub const KEY_RIGHTARROW: i32 = 0xae;
pub const KEY_LEFTARROW: i32 = 0xac;
pub const KEY_UPARROW: i32 = 0xad;
pub const KEY_DOWNARROW: i32 = 0xaf;
pub const KEY_ESCAPE: i32 = 27;
pub const KEY_ENTER: i32 = 13;
pub const KEY_TAB: i32 = 9;
pub const KEY_F1: i32 = 0x80 + 0x3b;
pub const KEY_F2: i32 = 0x80 + 0x3c;
pub const KEY_F3: i32 = 0x80 + 0x3d;
pub const KEY_F4: i32 = 0x80 + 0x3e;
pub const KEY_F5: i32 = 0x80 + 0x3f;
pub const KEY_F6: i32 = 0x80 + 0x40;
pub const KEY_F7: i32 = 0x80 + 0x41;
pub const KEY_F8: i32 = 0x80 + 0x42;
pub const KEY_F9: i32 = 0x80 + 0x43;
pub const KEY_F10: i32 = 0x80 + 0x44;
pub const KEY_F11: i32 = 0x80 + 0x57;
pub const KEY_F12: i32 = 0x80 + 0x58;
pub const KEY_BACKSPACE: i32 = 0x7f;
pub const KEY_PAUSE: i32 = 0xff;
pub const KEY_EQUALS: i32 = 0x3d;
pub const KEY_MINUS: i32 = 0x2d;
pub const KEY_RSHIFT: i32 = 0x80 + 0x36;
pub const KEY_RCTRL: i32 = 0x80 + 0x1d;
pub const KEY_RALT: i32 = 0x80 + 0x38;
pub const KEY_LALT: i32 = KEY_RALT;
pub const KEY_CAPSLOCK: i32 = 0x80 + 0x3a;
pub const KEY_NUMLOCK: i32 = 0x80 + 0x45;
pub const KEY_SCRLCK: i32 = 0x80 + 0x46;
pub const KEY_PRTSCR: i32 = 0x80 + 0x59;
pub const KEY_HOME: i32 = 0x80 + 0x47;
pub const KEY_END: i32 = 0x80 + 0x4f;
pub const KEY_PGUP: i32 = 0x80 + 0x49;
pub const KEY_PGDN: i32 = 0x80 + 0x51;
pub const KEY_INS: i32 = 0x80 + 0x52;
pub const KEY_DEL: i32 = 0x80 + 0x53;
pub const KEYP_0: i32 = KEY_INS;
pub const KEYP_1: i32 = KEY_END;
pub const KEYP_2: i32 = KEY_DOWNARROW;
pub const KEYP_3: i32 = KEY_PGDN;
pub const KEYP_4: i32 = KEY_LEFTARROW;
pub const KEYP_5: i32 = 0x80 + 0x4c;
pub const KEYP_6: i32 = KEY_RIGHTARROW;
pub const KEYP_7: i32 = KEY_HOME;
pub const KEYP_8: i32 = KEY_UPARROW;
pub const KEYP_9: i32 = KEY_PGUP;
pub const KEYP_DIVIDE: i32 = b'/' as i32;
pub const KEYP_PLUS: i32 = b'+' as i32;
pub const KEYP_MINUS: i32 = b'-' as i32;
pub const KEYP_MULTIPLY: i32 = b'*' as i32;
pub const KEYP_PERIOD: i32 = 0;
pub const KEYP_EQUALS: i32 = KEY_EQUALS;
pub const KEYP_ENTER: i32 = KEY_ENTER;
pub const KEY_NONUSBACKSLASH: i32 = b'\\' as i32;

pub const SCANCODE_TO_KEYS: [i32; 104] = [
    0,
    0,
    0,
    0,
    b'a' as i32,
    b'b' as i32,
    b'c' as i32,
    b'd' as i32,
    b'e' as i32,
    b'f' as i32,
    b'g' as i32,
    b'h' as i32,
    b'i' as i32,
    b'j' as i32,
    b'k' as i32,
    b'l' as i32,
    b'm' as i32,
    b'n' as i32,
    b'o' as i32,
    b'p' as i32,
    b'q' as i32,
    b'r' as i32,
    b's' as i32,
    b't' as i32,
    b'u' as i32,
    b'v' as i32,
    b'w' as i32,
    b'x' as i32,
    b'y' as i32,
    b'z' as i32,
    b'1' as i32,
    b'2' as i32,
    b'3' as i32,
    b'4' as i32,
    b'5' as i32,
    b'6' as i32,
    b'7' as i32,
    b'8' as i32,
    b'9' as i32,
    b'0' as i32,
    KEY_ENTER,
    KEY_ESCAPE,
    KEY_BACKSPACE,
    KEY_TAB,
    b' ' as i32,
    KEY_MINUS,
    KEY_EQUALS,
    b'[' as i32,
    b']' as i32,
    b'\\' as i32,
    0,
    b';' as i32,
    b'\'' as i32,
    b'`' as i32,
    b',' as i32,
    b'.' as i32,
    b'/' as i32,
    KEY_CAPSLOCK,
    KEY_F1,
    KEY_F2,
    KEY_F3,
    KEY_F4,
    KEY_F5,
    KEY_F6,
    KEY_F7,
    KEY_F8,
    KEY_F9,
    KEY_F10,
    KEY_F11,
    KEY_F12,
    KEY_PRTSCR,
    KEY_SCRLCK,
    KEY_PAUSE,
    KEY_INS,
    KEY_HOME,
    KEY_PGUP,
    KEY_DEL,
    KEY_END,
    KEY_PGDN,
    KEY_RIGHTARROW,
    KEY_LEFTARROW,
    KEY_DOWNARROW,
    KEY_UPARROW,
    KEY_NUMLOCK,
    KEYP_DIVIDE,
    KEYP_MULTIPLY,
    KEYP_MINUS,
    KEYP_PLUS,
    KEYP_ENTER,
    KEYP_1,
    KEYP_2,
    KEYP_3,
    KEYP_4,
    KEYP_5,
    KEYP_6,
    KEYP_7,
    KEYP_8,
    KEYP_9,
    KEYP_0,
    KEYP_PERIOD,
    KEY_NONUSBACKSLASH,
    0,
    0,
    KEYP_EQUALS,
];

pub fn short(value: u16) -> i16 {
    i16::from_le_bytes(value.to_le_bytes())
}

pub fn long(value: u32) -> i32 {
    i32::from_le_bytes(value.to_le_bytes())
}
