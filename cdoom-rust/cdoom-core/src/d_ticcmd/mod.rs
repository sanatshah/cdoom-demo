//! Network/demo tic command layout from `d_ticcmd.h`.

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct TicCmd {
    pub forwardmove: i8,
    pub sidemove: i8,
    pub angleturn: i16,
    pub chatchar: u8,
    pub buttons: u8,
    pub consistancy: u8,
    pub buttons2: u8,
    pub inventory: i32,
    pub lookfly: u8,
    pub arti: u8,
}
