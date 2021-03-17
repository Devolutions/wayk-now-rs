use crate::container::Vec8;
use crate::message::EdgeRect;
use alloc::vec::Vec;
use core::mem;

__flags_struct! {
    SurfaceResponseFlags: u8 => {
        failure = FAILURE = 0x80,
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum SurfaceMessageType {
    #[value = 0x01]
    ListReq,
    #[value = 0x02]
    ListRsp,
    #[value = 0x03]
    MapReq,
    #[value = 0x04]
    MapRsp,
    #[value = 0x05]
    SelectReq,
    #[value = 0x06]
    SelectRsp,
    #[fallback]
    Other(u8),
}

// NOW_SURFACE_DEF

__flags_struct! {
    SurfacePropertiesFlags: u16 => {
        primary = PRIMARY = 0x0001,
        mirrored = MIRRORED = 0x0002,
        disabled = DISABLED = 0x0004,
        selected = SELECTED = 0x0008,
    }
}

impl Default for SurfacePropertiesFlags {
    fn default() -> Self {
        Self {
            value: SurfacePropertiesFlags::SELECTED | SurfacePropertiesFlags::PRIMARY,
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum SurfaceOrientation {
    #[value = 0]
    Landscape,
    #[value = 90]
    Portrait,
    #[value = 180]
    LandscapeFlipped,
    #[value = 270]
    PortraitFlipped,
    #[fallback]
    Other(u16),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct NowSurfaceDef {
    size: u16,
    pub flags: SurfacePropertiesFlags,
    pub surface_id: u16,
    pub orientation: SurfaceOrientation,
    pub rect: EdgeRect,
    // unused fields
    #[decode_ignore]
    #[encode_ignore]
    dpi_x: u16,
    #[decode_ignore]
    #[encode_ignore]
    dpi_y: u16,
    #[decode_ignore]
    #[encode_ignore]
    pct_scale_x: u16,
    #[decode_ignore]
    #[encode_ignore]
    pct_scale_y: u16,
    #[decode_ignore]
    #[encode_ignore]
    native_rect: EdgeRect,
}

impl NowSurfaceDef {
    pub const REQUIRED_SIZE: usize = 16;

    pub fn new(surface_id: u16, rect: EdgeRect) -> Self {
        Self {
            size: Self::REQUIRED_SIZE as u16,
            flags: SurfacePropertiesFlags::default(),
            surface_id,
            orientation: SurfaceOrientation::Landscape,
            rect,
            dpi_x: 0,
            dpi_y: 0,
            pct_scale_x: 0,
            pct_scale_y: 0,
            native_rect: EdgeRect::default(),
        }
    }

    pub fn flags<F: Into<SurfacePropertiesFlags>>(self, flags: F) -> Self {
        Self {
            flags: flags.into(),
            ..self
        }
    }

    pub fn orientation<O: Into<SurfaceOrientation>>(self, orientation: O) -> Self {
        Self {
            orientation: orientation.into(),
            ..self
        }
    }
}

// NOW_SURFACE_MAP

#[derive(Debug, Clone, Encode, Decode)]
pub struct NowSurfaceMap {
    size: u16,
    flags: u16,
    pub surface_id: u16,
    pub output_id: u16,
    pub output_rect: EdgeRect,
}

impl NowSurfaceMap {
    pub const REQUIRED_SIZE: usize = mem::size_of::<Self>();

    pub fn new(surface_id: u16, output_id: u16, output_rect: EdgeRect) -> Self {
        Self {
            size: Self::REQUIRED_SIZE as u16,
            flags: 0,
            surface_id,
            output_id,
            output_rect,
        }
    }
}

// NOW_SURFACE_MSG

#[derive(Encode, Decode, Debug, Clone)]
#[meta_enum = "SurfaceMessageType"]
pub enum NowSurfaceMsg<'a> {
    ListReq(NowSurfaceListReqMsg),
    ListRsp(NowSurfaceListRspMsg),
    MapReq(NowSurfaceMapReqMsg),
    MapRsp(NowSurfaceMapRspMsg),
    SelectReq(NowSurfaceSelectReqMsg),
    SelectRsp(NowSurfaceSelectRspMsg),
    #[fallback]
    Custom(&'a [u8]),
}

impl From<NowSurfaceListReqMsg> for NowSurfaceMsg<'_> {
    fn from(msg: NowSurfaceListReqMsg) -> Self {
        Self::ListReq(msg)
    }
}

impl From<NowSurfaceListRspMsg> for NowSurfaceMsg<'_> {
    fn from(msg: NowSurfaceListRspMsg) -> Self {
        Self::ListRsp(msg)
    }
}

impl From<NowSurfaceMapReqMsg> for NowSurfaceMsg<'_> {
    fn from(msg: NowSurfaceMapReqMsg) -> Self {
        Self::MapReq(msg)
    }
}

impl From<NowSurfaceMapRspMsg> for NowSurfaceMsg<'_> {
    fn from(msg: NowSurfaceMapRspMsg) -> Self {
        Self::MapRsp(msg)
    }
}

impl From<NowSurfaceSelectReqMsg> for NowSurfaceMsg<'_> {
    fn from(msg: NowSurfaceSelectReqMsg) -> Self {
        Self::SelectReq(msg)
    }
}

impl From<NowSurfaceSelectRspMsg> for NowSurfaceMsg<'_> {
    fn from(msg: NowSurfaceSelectRspMsg) -> Self {
        Self::SelectRsp(msg)
    }
}

// subtypes

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowSurfaceListReqMsg {
    subtype: SurfaceMessageType,
    flags: u8,
    pub sequence_id: u16,
    pub desktop_width: u16,
    pub desktop_height: u16,
    pub surfaces: Vec8<NowSurfaceDef>,
}

impl NowSurfaceListReqMsg {
    pub const SUBTYPE: SurfaceMessageType = SurfaceMessageType::ListReq;
    pub const REQUIRED_SIZE: usize = 9;

    pub fn new(sequence_id: u16, desktop_width: u16, desktop_height: u16) -> Self {
        Self::new_with_surfaces(sequence_id, desktop_width, desktop_height, Vec::new())
    }

    pub fn new_with_surfaces(
        sequence_id: u16,
        desktop_width: u16,
        desktop_height: u16,
        surfaces: Vec<NowSurfaceDef>,
    ) -> Self {
        Self {
            subtype: SurfaceMessageType::ListReq,
            flags: 0,
            sequence_id,
            desktop_width,
            desktop_height,
            surfaces: Vec8(surfaces),
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowSurfaceListRspMsg {
    subtype: SurfaceMessageType,
    pub flags: SurfaceResponseFlags,
    pub sequence_id: u16,
}

impl NowSurfaceListRspMsg {
    pub const SUBTYPE: SurfaceMessageType = SurfaceMessageType::ListRsp;

    pub fn new(flags: SurfaceResponseFlags, sequence_id: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            sequence_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowSurfaceMapReqMsg {
    subtype: SurfaceMessageType,
    pub flags: u8, // TODO: find flags values
    pub sequence_id: u16,
    pub desktop_width: u16,
    pub desktop_height: u16,
    pub maps: Vec8<NowSurfaceMap>,
}

impl NowSurfaceMapReqMsg {
    pub const SUBTYPE: SurfaceMessageType = SurfaceMessageType::MapReq;
    pub const REQUIRED_SIZE: usize = 9;

    pub fn new(sequence_id: u16, desktop_width: u16, desktop_height: u16) -> Self {
        Self::new_with_mappings(sequence_id, desktop_width, desktop_height, Vec::new())
    }

    pub fn new_with_mappings(
        sequence_id: u16,
        desktop_width: u16,
        desktop_height: u16,
        maps: Vec<NowSurfaceMap>,
    ) -> Self {
        Self {
            subtype: SurfaceMessageType::ListReq,
            flags: 0,
            sequence_id,
            desktop_width,
            desktop_height,
            maps: Vec8(maps),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct NowSurfaceMapRspMsg {
    subtype: SurfaceMessageType,
    pub flags: SurfaceResponseFlags,
    pub sequence_id: u16,
}

impl NowSurfaceMapRspMsg {
    pub const SUBTYPE: SurfaceMessageType = SurfaceMessageType::MapRsp;

    pub fn new(flags: SurfaceResponseFlags, sequence_id: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            sequence_id,
        }
    }
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct NowSurfaceSelectReqMsg {
    subtype: SurfaceMessageType,
    pub flags: u8, // TODO: find flags values
    pub sequence_id: u16,
    reserved: u16,
    pub surface_id: u16,
}

impl NowSurfaceSelectReqMsg {
    pub const SUBTYPE: SurfaceMessageType = SurfaceMessageType::SelectReq;

    pub fn new(flags: u8, sequence_id: u16, surface_id: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            sequence_id,
            reserved: 0,
            surface_id,
        }
    }
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct NowSurfaceSelectRspMsg {
    subtype: SurfaceMessageType,
    pub flags: SurfaceResponseFlags,
    pub sequence_id: u16,
}

impl NowSurfaceSelectRspMsg {
    pub const SUBTYPE: SurfaceMessageType = SurfaceMessageType::SelectRsp;

    pub fn new(flags: SurfaceResponseFlags, sequence_id: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            sequence_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    #[rustfmt::skip]
    const SURFACE_LIST_REQ_MSG: [u8; 25] = [
        0x01, // subtype
        0x00, // flags
        0x00, 0x00, // sequence id
        0x00, 0x04, // desktop width
        0x00, 0x03, // desktop height
        0x01, // surface count
        // surface(s)
        0x10, 0x00, // size
        0x09, 0x00, // flags
        0x00, 0x00, // surface id
        0x00, 0x00, // orientation
        0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x03, // rect
    ];

    #[test]
    fn decoding_with_subtype_check() {
        let msg = NowSurfaceMsg::decode(&SURFACE_LIST_REQ_MSG).unwrap();
        if let NowSurfaceMsg::ListReq(msg) = msg {
            assert_eq!(msg.subtype, SurfaceMessageType::ListReq);
            assert_eq!(msg.sequence_id, 0);
            assert_eq!(msg.desktop_width, 1024);
            assert_eq!(msg.desktop_height, 768);
            assert_eq!(msg.surfaces.len(), 1);
            let surface = &msg.surfaces[0];
            assert_eq!(surface.size, 16);
            assert_eq!(surface.flags, SurfacePropertiesFlags::default());
            assert_eq!(surface.surface_id, 0);
            assert_eq!(surface.orientation, SurfaceOrientation::Landscape);
            let rect = &surface.rect;
            assert_eq!(rect.left, 0);
            assert_eq!(rect.top, 0);
            assert_eq!(rect.right, 1024);
            assert_eq!(rect.bottom, 768);
        } else {
            panic!("expected a surface list req message and got {:?}", msg);
        }
    }

    #[test]
    fn list_req_encoding() {
        let rect = EdgeRect {
            left: 0,
            top: 0,
            right: 1024,
            bottom: 768,
        };
        let surface = NowSurfaceDef::new(0, rect);
        let msg = NowSurfaceListReqMsg::new_with_surfaces(0, 1024, 768, vec![surface]);
        assert_eq!(msg.encode().unwrap(), SURFACE_LIST_REQ_MSG.to_vec());
    }

    // TODO: test NowSurfaceMapReqMsg
}
