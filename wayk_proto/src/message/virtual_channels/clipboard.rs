// Clipboard

use crate::{
    container::{Bytes32, Vec32, Vec8},
    message::NowString256,
};
use num_derive::FromPrimitive;

#[derive(Encode, Decode, FromPrimitive, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ClipboardMessageType {
    CapabilitiesReq = 0x01,
    CapabilitiesRsp = 0x02,
    ControlReq = 0x03,
    ControlRsp = 0x04,
    SuspendReq = 0x05,
    SuspendRsp = 0x06,
    ResumeReq = 0x07,
    ResumeRsp = 0x08,
    FormatListReq = 0x09,
    FormatListRsp = 0x0A,
    FormatDataReq = 0x0B,
    FormatDataRsp = 0x0C,
}

#[derive(Encode, Decode, FromPrimitive, Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum ClipboardControlState {
    None = 0x0000,
    Auto = 0x0001,
    Manual = 0x0002,
}

__flags_struct! {
    ClipboardResponseFlags: u8 => {
        failure = FAILURE = 0x80,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct ClipboardFormatDef {
    pub id: u32,
    pub name: NowString256,
}

impl ClipboardFormatDef {
    pub fn new(id: u32, name: NowString256) -> Self {
        Self { id, name }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "ClipboardMessageType"]
pub enum NowClipboardMsg<'a> {
    CapabilitiesReq(NowClipboardCapabilitiesReqMsg),
    CapabilitiesRsp(NowClipboardCapabilitiesRspMsg),
    ControlReq(NowClipboardControlReqMsg),
    ControlRsp(NowClipboardControlRspMsg),
    SuspendReq(NowClipboardSuspendReqMsg),
    SuspendRsp(NowClipboardSuspendRspMsg),
    ResumeReq(NowClipboardResumeReqMsg),
    ResumeRsp(NowClipboardResumeRspMsg),
    FormatListReq(NowClipboardFormatListReqMsg),
    FormatListRsp(NowClipboardFormatListRspMsg),
    FormatDataReq(NowClipboardFormatDataReqMsg),
    FormatDataRsp(NowClipboardFormatDataRspMsg<'a>),

    #[decode_ignore]
    FormatDataRspOwned(NowClipboardFormatDataRspMsgOwned),
}

impl From<NowClipboardCapabilitiesReqMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardCapabilitiesReqMsg) -> Self {
        Self::CapabilitiesReq(msg)
    }
}

impl From<NowClipboardCapabilitiesRspMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardCapabilitiesRspMsg) -> Self {
        Self::CapabilitiesRsp(msg)
    }
}

impl From<NowClipboardControlReqMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardControlReqMsg) -> Self {
        Self::ControlReq(msg)
    }
}

impl From<NowClipboardControlRspMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardControlRspMsg) -> Self {
        Self::ControlRsp(msg)
    }
}

impl From<NowClipboardSuspendReqMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardSuspendReqMsg) -> Self {
        Self::SuspendReq(msg)
    }
}

impl From<NowClipboardSuspendRspMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardSuspendRspMsg) -> Self {
        Self::SuspendRsp(msg)
    }
}

impl From<NowClipboardResumeReqMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardResumeReqMsg) -> Self {
        Self::ResumeReq(msg)
    }
}

impl From<NowClipboardResumeRspMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardResumeRspMsg) -> Self {
        Self::ResumeRsp(msg)
    }
}

impl From<NowClipboardFormatListReqMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardFormatListReqMsg) -> Self {
        Self::FormatListReq(msg)
    }
}

impl From<NowClipboardFormatListRspMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardFormatListRspMsg) -> Self {
        Self::FormatListRsp(msg)
    }
}

impl From<NowClipboardFormatDataReqMsg> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardFormatDataReqMsg) -> Self {
        Self::FormatDataReq(msg)
    }
}

impl<'a> From<NowClipboardFormatDataRspMsg<'a>> for NowClipboardMsg<'a> {
    fn from(msg: NowClipboardFormatDataRspMsg<'a>) -> Self {
        Self::FormatDataRsp(msg)
    }
}

impl From<NowClipboardFormatDataRspMsgOwned> for NowClipboardMsg<'_> {
    fn from(msg: NowClipboardFormatDataRspMsgOwned) -> Self {
        Self::FormatDataRspOwned(msg)
    }
}

// subtypes

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardCapabilitiesReqMsg {
    subtype: ClipboardMessageType,
    flags: u8,
    capabilities: u16,
}

impl Default for NowClipboardCapabilitiesReqMsg {
    fn default() -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            capabilities: 0,
        }
    }
}

impl NowClipboardCapabilitiesReqMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::CapabilitiesReq;

    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardCapabilitiesRspMsg {
    subtype: ClipboardMessageType,
    pub flags: ClipboardResponseFlags,
    capabilities: u16,
}

impl Default for NowClipboardCapabilitiesRspMsg {
    fn default() -> Self {
        Self::new_with_flags(ClipboardResponseFlags::new_empty())
    }
}

impl NowClipboardCapabilitiesRspMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::CapabilitiesRsp;

    pub fn new_with_flags(flags: ClipboardResponseFlags) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            capabilities: 0,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardControlReqMsg {
    subtype: ClipboardMessageType,
    flags: u8,
    pub control_state: ClipboardControlState,
}

impl NowClipboardControlReqMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::ControlReq;

    pub fn new(control_state: ClipboardControlState) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            control_state,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardControlRspMsg {
    subtype: ClipboardMessageType,
    pub flags: ClipboardResponseFlags,
    pub control_state: ClipboardControlState,
}

impl NowClipboardControlRspMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::ControlRsp;

    pub fn new(control_state: ClipboardControlState) -> Self {
        Self::new_with_flags(control_state, ClipboardResponseFlags::new_empty())
    }

    pub fn new_with_flags(control_state: ClipboardControlState, flags: ClipboardResponseFlags) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            control_state,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardSuspendReqMsg {
    subtype: ClipboardMessageType,
    flags: u8,
    reserved: u16,
}

impl Default for NowClipboardSuspendReqMsg {
    fn default() -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
        }
    }
}

impl NowClipboardSuspendReqMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::SuspendReq;
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardSuspendRspMsg {
    subtype: ClipboardMessageType,
    pub flags: ClipboardResponseFlags,
    reserved: u16,
}

impl Default for NowClipboardSuspendRspMsg {
    fn default() -> Self {
        Self::new_with_flags(ClipboardResponseFlags::new_empty())
    }
}

impl NowClipboardSuspendRspMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::SuspendRsp;

    pub fn new_with_flags(flags: ClipboardResponseFlags) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            reserved: 0,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardResumeReqMsg {
    subtype: ClipboardMessageType,
    flags: u8,
    reserved: u16,
}

impl Default for NowClipboardResumeReqMsg {
    fn default() -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
        }
    }
}

impl NowClipboardResumeReqMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::ResumeReq;
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardResumeRspMsg {
    subtype: ClipboardMessageType,
    pub flags: ClipboardResponseFlags,
    reserved: u16,
}

impl Default for NowClipboardResumeRspMsg {
    fn default() -> Self {
        Self::new_with_flags(ClipboardResponseFlags::new_empty())
    }
}

impl NowClipboardResumeRspMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::ResumeRsp;

    pub fn new_with_flags(flags: ClipboardResponseFlags) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            reserved: 0,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardFormatListReqMsg {
    subtype: ClipboardMessageType,
    flags: u8,
    pub sequence_id: u16,
    pub formats: Vec8<ClipboardFormatDef>,
}

impl NowClipboardFormatListReqMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::FormatListReq;

    pub fn new(sequence_id: u16) -> Self {
        Self::new_with_formats(sequence_id, Vec::new())
    }

    pub fn new_with_formats(sequence_id: u16, formats: Vec<ClipboardFormatDef>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            sequence_id,
            formats: Vec8(formats),
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardFormatListRspMsg {
    subtype: ClipboardMessageType,
    pub flags: ClipboardResponseFlags,
    pub sequence_id: u16,
}

impl NowClipboardFormatListRspMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::FormatListRsp;

    pub fn new(sequence_id: u16) -> Self {
        Self::new_with_flags(sequence_id, ClipboardResponseFlags::new_empty())
    }

    pub fn new_with_flags(sequence_id: u16, flags: ClipboardResponseFlags) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            sequence_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardFormatDataReqMsg {
    subtype: ClipboardMessageType,
    flags: u8,
    pub sequence_id: u16,
    pub format_id: u32,
}

impl NowClipboardFormatDataReqMsg {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::FormatDataReq;

    pub fn new(sequence_id: u16, format_id: u32) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            sequence_id,
            format_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardFormatDataRspMsg<'a> {
    subtype: ClipboardMessageType,
    pub flags: ClipboardResponseFlags,
    pub sequence_id: u16,
    pub format_id: u32,
    pub format_data: Bytes32<'a>,
}

impl<'a> NowClipboardFormatDataRspMsg<'a> {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::FormatDataRsp;

    pub fn new(sequence_id: u16, format_id: u32) -> Self {
        Self::new_with_flags(sequence_id, format_id, ClipboardResponseFlags::new_empty())
    }

    pub fn new_with_flags(sequence_id: u16, format_id: u32, flags: ClipboardResponseFlags) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            sequence_id,
            format_id,
            format_data: Bytes32(&[]),
        }
    }

    pub fn new_with_format_data(sequence_id: u16, format_id: u32, format_data: &'a [u8]) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: ClipboardResponseFlags::new_empty(),
            sequence_id,
            format_id,
            format_data: Bytes32(format_data),
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowClipboardFormatDataRspMsgOwned {
    subtype: ClipboardMessageType,
    pub flags: ClipboardResponseFlags,
    pub sequence_id: u16,
    pub format_id: u32,
    pub format_data: Vec32<u8>,
}

impl NowClipboardFormatDataRspMsgOwned {
    pub const SUBTYPE: ClipboardMessageType = ClipboardMessageType::FormatDataRsp;

    pub fn new(sequence_id: u16, format_id: u32) -> Self {
        Self::new_with_flags(sequence_id, format_id, ClipboardResponseFlags::new_empty())
    }

    pub fn new_with_flags(sequence_id: u16, format_id: u32, flags: ClipboardResponseFlags) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            sequence_id,
            format_id,
            format_data: Vec32(Vec::new()),
        }
    }

    pub fn new_with_format_data(sequence_id: u16, format_id: u32, format_data: Vec<u8>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: ClipboardResponseFlags::new_empty(),
            sequence_id,
            format_id,
            format_data: Vec32(format_data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        message::{ChannelName, NowBody, NowVirtualChannel, VirtChannelsCtx},
        packet::NowPacket,
        serialization::{Decode, Encode},
    };
    use std::io::Cursor;

    fn get_ctx() -> VirtChannelsCtx {
        let mut vchan_ctx = VirtChannelsCtx::new();
        vchan_ctx.insert(0x00, ChannelName::Clipboard);
        vchan_ctx
    }

    #[rustfmt::skip]
    const NOW_CLIPBOARD_CAPS_REQ_WITH_HEADER: [u8; 8] = [
        // vheader
        0x04, 0x00, 0x00, 0x81,
        // clipboard
        0x01, // subtype
        0x00, // flags
        0x00, 0x00 // capabilities
    ];

    #[test]
    fn clipboard_caps_req_decoding() {
        let mut buffer = Vec::new();
        let mut reader = Cursor::new(&NOW_CLIPBOARD_CAPS_REQ_WITH_HEADER[..]);
        match NowPacket::read_from(&mut reader, &mut buffer, &get_ctx()) {
            Ok(packet) => match packet.body {
                NowBody::Message(_) => panic!("decoded a now message from a virtual channel packet"),
                NowBody::VirtualChannel(vchan) => {
                    if let NowVirtualChannel::Clipboard(NowClipboardMsg::CapabilitiesReq(msg)) = vchan {
                        assert_eq!(msg.subtype, ClipboardMessageType::CapabilitiesReq);
                        assert_eq!(msg.flags, 0x00);
                        assert_eq!(msg.capabilities, 0x0000);
                    } else {
                        panic!("decoded wrong virtual channel message");
                    }
                }
            },
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode capabilities packet");
            }
        }
    }

    #[test]
    fn clipboard_caps_req_encoding() {
        let msg = NowClipboardCapabilitiesReqMsg::default();
        let channel_id = get_ctx().get_id_by_channel(&ChannelName::Clipboard).unwrap();
        let packet = NowPacket::from_virt_channel(NowClipboardMsg::from(msg), channel_id);
        assert_eq!(packet.encode().unwrap(), NOW_CLIPBOARD_CAPS_REQ_WITH_HEADER.to_vec());
    }

    #[rustfmt::skip]
    const CLIPBOARD_CONTROL_RSP: [u8; 4] = [0x04, 0x00, 0x01, 0x00];

    #[test]
    fn clipboard_ctrl_rsp_decoding() {
        let msg = NowClipboardControlRspMsg::decode(&CLIPBOARD_CONTROL_RSP).unwrap();
        assert_eq!(msg.subtype, ClipboardMessageType::ControlRsp);
        assert_eq!(msg.flags, 0x00);
        assert_eq!(msg.control_state, ClipboardControlState::Auto);
    }

    #[test]
    fn clipboard_ctrl_rsp_encoding() {
        let msg = NowClipboardControlRspMsg::new(ClipboardControlState::Auto);
        assert_eq!(msg.encode().unwrap(), CLIPBOARD_CONTROL_RSP.to_vec());
    }
}
