//! C-compatible foundational types and constants.

pub type Boolean = i32;
pub type Byte = u8;
pub type Pixel = u8;
pub type DPixel = i16;
pub type Fixed = i32;
pub type Angle = u32;

pub const FALSE: Boolean = 0;
pub const TRUE: Boolean = 1;

pub const FRACBITS: i32 = 16;
pub const FRACUNIT: Fixed = 1 << FRACBITS;

pub const FINEANGLES: usize = 8192;
pub const FINEMASK: usize = FINEANGLES - 1;
pub const ANGLETOFINESHIFT: u32 = 19;

pub const ANG45: Angle = 0x20000000;
pub const ANG90: Angle = 0x40000000;
pub const ANG180: Angle = 0x80000000;
pub const ANG270: Angle = 0xc0000000;
pub const ANG_MAX: Angle = 0xffffffff;

pub const ANG1: Angle = ANG45 / 45;
pub const ANG60: Angle = ANG180 / 3;
pub const ANG1_X: Angle = 0x01000000;

pub const SLOPERANGE: i32 = 2048;
pub const SLOPEBITS: i32 = 11;
pub const DBITS: i32 = FRACBITS - SLOPEBITS;

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
