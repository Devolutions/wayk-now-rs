// NOW_SIZE_RECT

use core::mem;

#[derive(Decode, Encode, Debug, Clone, Default)]
pub struct SizeRect {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl SizeRect {
    pub const REQUIRED_SIZE: usize = mem::size_of::<Self>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    #[rustfmt::skip]
    const NOW_SIZE_RECT: [u8; 8] = [
        0x60, 0x07, // X pos
        0x24, 0x04, // Y pos
        0x0c, 0x00, // width
        0x0c, 0x00, // height
    ];

    #[test]
    fn decoding() {
        let rect = SizeRect::decode(&NOW_SIZE_RECT).unwrap();
        assert_eq!(rect.x, 1888);
        assert_eq!(rect.y, 1060);
        assert_eq!(rect.width, 12);
        assert_eq!(rect.height, 12);
    }

    #[test]
    fn encoding() {
        let rect = SizeRect {
            x: 1888,
            y: 1060,
            width: 12,
            height: 12,
        };
        assert_eq!(rect.encode().unwrap(), NOW_SIZE_RECT.to_vec());
    }
}
