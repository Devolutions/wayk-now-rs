// NOW_MOUSE_MSG

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum MouseMessageType {
    #[value = 0x01]
    Position,
    #[value = 0x02]
    Cursor,
    #[value = 0x03]
    Mode,
    #[value = 0x04]
    State,
    #[fallback]
    Other(u8),
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

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum MouseCursorType {
    #[value = 0x00]
    Mono,
    #[value = 0x01]
    Color,
    #[value = 0x02]
    Alpha,
    #[fallback]
    Other(u8),
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum MouseMode {
    #[value = 0x01]
    Primary,
    #[value = 0x02]
    Secondary,
    #[value = 0x03]
    Disabled,
    #[fallback]
    Other(u8),
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum MouseState {
    #[value = 0x01]
    Primary,
    #[value = 0x02]
    Secondary,
    #[value = 0x03]
    Disabled,
    #[fallback]
    Other(u8),
}
