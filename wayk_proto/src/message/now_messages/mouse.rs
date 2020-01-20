// NOW_MOUSE_MSG

use num_derive::FromPrimitive;

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum MouseMessageType {
    Position = 0x01,
    Cursor = 0x02,
    Mode = 0x03,
    State = 0x04,
}

__flags_struct! {
    MousePositionFlags: u8 => {
        same = SAME = 0x01,
    }
}

__flags_struct! {
    MouseCursorFlags: u8 => {
        large = LARGE = 0x01,
    }
}

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum MouseCursorType {
    Mono = 0x00,
    Color = 0x01,
    Alpha = 0x02,
}

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum MouseMode {
    Primary = 0x01,
    Secondary = 0x02,
    Disabled = 0x03,
}

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum MouseState {
    Primary = 0x01,
    Secondary = 0x02,
    Disabled = 0x03,
}
