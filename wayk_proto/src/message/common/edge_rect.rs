// NOW_EDGE_RECT

use core::mem;

#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct EdgeRect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

impl EdgeRect {
    pub const REQUIRED_SIZE: usize = mem::size_of::<Self>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    #[rustfmt::skip]
    const NOW_EDGE_RECT: [u8; 8] = [
        0x00, 0x00, // left
        0x00, 0x00, // top
        0x00, 0x04, // right
        0x00, 0x03, // bottom
    ];

    #[test]
    fn decoding() {
        let rect = EdgeRect::decode(&NOW_EDGE_RECT).unwrap();
        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 1024);
        assert_eq!(rect.bottom, 768);
    }

    #[test]
    fn encoding() {
        let rect = EdgeRect {
            left: 0,
            top: 0,
            right: 1024,
            bottom: 768,
        };
        assert_eq!(rect.encode().unwrap(), NOW_EDGE_RECT.to_vec());
    }
}
