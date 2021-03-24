// NOW_UPDATE_MSG

use crate::container::{Bytes32, Vec8};
use crate::message::{common, Codec, SizeRect};

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum UpdateMessageType {
    #[value = 0x01]
    UpdateGraphics,
    #[value = 0x02]
    UpdateRefresh,
    #[value = 0x03]
    UpdateSuppress,
    #[fallback]
    Other(u8),
}

__flags_struct! {
    UpdateGraphicsFlags: u32 => {
        frame_first = FRAME_FIRST = 0x0000_0001,
        frame_last = FRAME_LAST = 0x0000_0002,
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum UpdateRegionFlag {
    #[value = 0x01]
    Null,
    #[value = 0x02]
    Full,
    #[fallback]
    Other(u8),
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowUpdateRegion {
    pub surface_id: u16,
    pub flags: UpdateRegionFlag,
    pub rects: Vec8<SizeRect>,
}

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "UpdateMessageType"]
pub enum NowUpdateMsg<'a> {
    UpdateGraphics(NowUpdateGraphicsMsg<'a>),
    UpdateRefresh(NowUpdateRefreshMsg),
    UpdateSuppress(NowUpdateSuppressMsg),
    #[fallback]
    Custom(&'a [u8]),
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowUpdateGraphicsMsg<'a> {
    pub subtype: UpdateMessageType,
    flags: u8,

    pub codec_id: Codec,
    pub surface_id: u16,
    pub frame_id: u16,
    pub update_flags: UpdateGraphicsFlags,
    pub update_rect: common::SizeRect,
    pub update_data: Bytes32<'a>,
}

impl<'a> NowUpdateGraphicsMsg<'a> {
    pub const REQUIRED_SIZE: usize = 24;
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowUpdateRefreshMsg {
    pub subtype: UpdateMessageType,
    flags: u8,

    reserved: u8,
    pub regions: Vec8<NowUpdateRegion>,
}

#[derive(Decode, Encode, Debug, Clone)]
#[repr(C)]
pub struct NowUpdateSuppressMsg {
    pub subtype: UpdateMessageType,
    flags: u8,

    reserved: u8,
    pub regions: Vec8<NowUpdateRegion>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::{AbstractNowHeader, NowHeader};
    use crate::serialization::Decode;

    #[rustfmt::skip]
    const WAYK_NOW_UPDATE_GRAPHIC_MSG: [u8; 35] = [
        // header
        0x1d, 0x03, 0x00, 0x00, // msgSize
        0x00, // msgFlags
        0x42, // msgType
        // update graphic
        0x01, // subtype
        0x00, // flags
        0x02, 0x00, // codecId
        0x00, 0x00, // surfaceID
        0x01, 0x00, // frameID
        0x03, 0x00, 0x00, 0x00, // updateFlags
        0x60, 0x07, /* X pos */ 0x24, 0x04, /* Y pos */ 0x0c, 0x00,
        /* width */ 0x0c, 0x00, /* height */
        // updateRect
        0x05, 0x00, 0x00, 0x00, // updateSize
      	0x01, 0x02, 0x03, 0x04, 0x05,
    ];

    #[test]
    fn update_graphics_decoding() {
        let header = NowHeader::decode(&WAYK_NOW_UPDATE_GRAPHIC_MSG).unwrap();
        let update_graphic_payload = &WAYK_NOW_UPDATE_GRAPHIC_MSG[header.len()..];
        let ugm = NowUpdateGraphicsMsg::decode(update_graphic_payload).unwrap();
        assert_eq!(ugm.flags, 0x00);
        assert_eq!(ugm.codec_id, Codec::JPEG);
        assert_eq!(ugm.surface_id, 0x0000);
        assert_eq!(ugm.frame_id, 0x0001);
        assert_eq!(ugm.update_flags.value, 0x00000003);
        assert_eq!(ugm.update_data.len(), 5);
        assert_eq!(ugm.update_data[0], 0x01);
    }
}
