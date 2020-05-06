// File Transfer
use crate::{
    container::Bytes32,
    message::{
        status::{FileTransferStatusCode, NowStatus},
        NowString65535,
    },
};
use num_derive::FromPrimitive;

#[derive(Encode, Decode, FromPrimitive, Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum FileTransferMessageType {
    CapsetReq = 0x00,
    CapsetRsp = 0x01,
    AbortMsg = 0x02,
    CreateReq = 0x10,
    CreateRsp = 0x11,
    RetryReq = 0x12,
    RetryRsp = 0x13,
    SuspendReq = 0x14,
    SuspendRsp = 0x15,
    ResumeReq = 0x16,
    ResumeRsp = 0x17,
    CancelReq = 0x18,
    CancelRsp = 0x19,
    CompleteReq = 0x1A,
    CompleteRsp = 0x1B,
    Data = 0x20,
}

#[derive(Encode, Decode, Debug, Clone)]
#[meta_enum = "FileTransferMessageType"]
pub enum NowFileTransferMsg<'a> {
    CapsetReq(NowFileTransferCapsetReqMsg),
    CapsetRsp(NowFileTransferCapsetRspMsg),
    AbortMsg(NowFileTransferAbortMsgMsg),
    CreateReq(NowFileTransferCreateReqMsg),
    CreateRsp(NowFileTransferCreateRspMsg),
    RetryReq(NowFileTransferRetryReqMsg),
    RetryRsp(NowFileTransferRetryRspMsg),
    SuspendReq(NowFileTransferSuspendReqMsg),
    SuspendRsp(NowFileTransferSuspendRspMsg),
    ResumeReq(NowFileTransferResumeReqMsg),
    ResumeRsp(NowFileTransferResumeRspMsg),
    CancelReq(NowFileTransferCancelReqMsg),
    CancelRsp(NowFileTransferCancelRspMsg),
    CompleteReq(NowFileTransferCompleteReqMsg),
    CompleteRsp(NowFileTransferCompleteRspMsg),
    Data(NowFileTransferDataMsg<'a>),
}

impl From<NowFileTransferCapsetReqMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferCapsetReqMsg) -> Self {
        Self::CapsetReq(msg)
    }
}

impl From<NowFileTransferCapsetRspMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferCapsetRspMsg) -> Self {
        Self::CapsetRsp(msg)
    }
}

impl From<NowFileTransferAbortMsgMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferAbortMsgMsg) -> Self {
        Self::AbortMsg(msg)
    }
}

impl From<NowFileTransferCreateReqMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferCreateReqMsg) -> Self {
        Self::CreateReq(msg)
    }
}

impl From<NowFileTransferCreateRspMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferCreateRspMsg) -> Self {
        Self::CreateRsp(msg)
    }
}

impl From<NowFileTransferRetryReqMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferRetryReqMsg) -> Self {
        Self::RetryReq(msg)
    }
}

impl From<NowFileTransferRetryRspMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferRetryRspMsg) -> Self {
        Self::RetryRsp(msg)
    }
}

impl From<NowFileTransferSuspendReqMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferSuspendReqMsg) -> Self {
        Self::SuspendReq(msg)
    }
}

impl From<NowFileTransferSuspendRspMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferSuspendRspMsg) -> Self {
        Self::SuspendRsp(msg)
    }
}

impl From<NowFileTransferResumeReqMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferResumeReqMsg) -> Self {
        Self::ResumeReq(msg)
    }
}

impl From<NowFileTransferResumeRspMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferResumeRspMsg) -> Self {
        Self::ResumeRsp(msg)
    }
}

impl From<NowFileTransferCancelReqMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferCancelReqMsg) -> Self {
        Self::CancelReq(msg)
    }
}

impl From<NowFileTransferCancelRspMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferCancelRspMsg) -> Self {
        Self::CancelRsp(msg)
    }
}

impl From<NowFileTransferCompleteReqMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferCompleteReqMsg) -> Self {
        Self::CompleteReq(msg)
    }
}

impl From<NowFileTransferCompleteRspMsg> for NowFileTransferMsg<'_> {
    fn from(msg: NowFileTransferCompleteRspMsg) -> Self {
        Self::CompleteRsp(msg)
    }
}

impl<'a> From<NowFileTransferDataMsg<'a>> for NowFileTransferMsg<'a> {
    fn from(msg: NowFileTransferDataMsg<'a>) -> Self {
        Self::Data(msg)
    }
}

__flags_struct! {
    CompressionFlags: u32 => {
        compression_lz4 = COMPRESSION_LZ4 = 0x01, // Compression LZ4 is supported
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferCapsetReqMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    capabilities: u32,
    chunk_size: u32,
    compression_flags: CompressionFlags,
}

impl NowFileTransferCapsetReqMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::CapsetReq;

    pub fn new(session_id: u16, capabilities: u32, chunk_size: u32, compression_flags: CompressionFlags) -> Self {
        Self::new_with_capabilities_chunk_size_and_compression_flags(
            session_id,
            capabilities,
            chunk_size,
            compression_flags,
        )
    }

    pub fn new_with_capabilities_chunk_size_and_compression_flags(
        session_id: u16,
        capabilities: u32,
        chunk_size: u32,
        compression_flags: CompressionFlags,
    ) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            capabilities,
            chunk_size,
            compression_flags,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferCapsetRspMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    capabilities: u32,
    chunk_size: u32,
    compression_flags: CompressionFlags,
}

impl NowFileTransferCapsetRspMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::CapsetReq;

    pub fn new(session_id: u16, capabilities: u32, chunk_size: u32, compression_flags: CompressionFlags) -> Self {
        Self::new_with_capabilities_chunk_size_and_compression_flags(
            session_id,
            capabilities,
            chunk_size,
            compression_flags,
        )
    }

    pub fn new_with_capabilities_chunk_size_and_compression_flags(
        session_id: u16,
        capabilities: u32,
        chunk_size: u32,
        compression_flags: CompressionFlags,
    ) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            capabilities,
            chunk_size,
            compression_flags,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferAbortMsgMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    status: NowStatus<FileTransferStatusCode>,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferCreateReqMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    file_size: u64,
    file_name: NowString65535,
}

impl NowFileTransferCreateReqMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::CreateReq;

    pub fn new(session_id: u16, file_size: u64, file_name: NowString65535) -> Self {
        Self::new_with_file_name_and_file_size(session_id, file_size, file_name)
    }

    pub fn new_with_file_name_and_file_size(session_id: u16, file_size: u64, file_name: NowString65535) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            file_size,
            file_name,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferCreateRspMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    status: NowStatus<FileTransferStatusCode>,
}

impl NowFileTransferCreateRspMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::CreateRsp;

    pub fn new(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self::new_with_status(session_id, status)
    }

    pub fn new_with_status(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            status,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferRetryReqMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    file_size: u64,
    file_name: NowString65535,
}

impl NowFileTransferRetryReqMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::RetryReq;

    pub fn new(session_id: u16, file_size: u64, file_name: NowString65535) -> Self {
        Self::new_with_file_name_and_file_size(session_id, file_size, file_name)
    }

    pub fn new_with_file_name_and_file_size(session_id: u16, file_size: u64, file_name: NowString65535) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            file_size,
            file_name,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferRetryRspMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    status: NowStatus<FileTransferStatusCode>,
}

impl NowFileTransferRetryRspMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::RetryRsp;

    pub fn new(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self::new_with_status(session_id, status)
    }

    pub fn new_with_status(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            status,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferSuspendReqMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,
}

impl NowFileTransferSuspendReqMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::SuspendReq;

    pub fn new(session_id: u16) -> Self {
        Self::new_with_session_id(session_id)
    }

    pub fn new_with_session_id(session_id: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferSuspendRspMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    status: NowStatus<FileTransferStatusCode>,
}

impl NowFileTransferSuspendRspMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::SuspendRsp;

    pub fn new(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self::new_with_status(session_id, status)
    }

    pub fn new_with_status(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            status,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferResumeReqMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,
}

impl NowFileTransferResumeReqMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::ResumeReq;

    pub fn new(session_id: u16) -> Self {
        Self::new_with_session_id(session_id)
    }

    pub fn new_with_session_id(session_id: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferResumeRspMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    status: NowStatus<FileTransferStatusCode>,
}

impl NowFileTransferResumeRspMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::ResumeRsp;

    pub fn new(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self::new_with_status(session_id, status)
    }

    pub fn new_with_status(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            status,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferCancelReqMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,
}

impl NowFileTransferCancelReqMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::CancelReq;

    pub fn new(session_id: u16) -> Self {
        Self::new_with_session_id(session_id)
    }

    pub fn new_with_session_id(session_id: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferCancelRspMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    status: NowStatus<FileTransferStatusCode>,
}

impl NowFileTransferCancelRspMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::CancelRsp;

    pub fn new(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self::new_with_status(session_id, status)
    }

    pub fn new_with_status(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            status,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferCompleteReqMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,
}

impl NowFileTransferCompleteReqMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::CompleteReq;

    pub fn new(session_id: u16) -> Self {
        Self::new_with_session_id(session_id)
    }

    pub fn new_with_session_id(session_id: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferCompleteRspMsg {
    subtype: FileTransferMessageType,
    flags: u8,
    session_id: u16,

    status: NowStatus<FileTransferStatusCode>,
}

impl NowFileTransferCompleteRspMsg {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::CompleteRsp;

    pub fn new(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self::new_with_status(session_id, status)
    }

    pub fn new_with_status(session_id: u16, status: NowStatus<FileTransferStatusCode>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            session_id,
            status,
        }
    }
}

__flags_struct! {
    FlagsCompressed: u8 => {
        flag_compressed = NOW_FILE_TRANSFER_FLAG_COMPRESSED = 0x80, // The data is compressed
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowFileTransferDataMsg<'a> {
    subtype: FileTransferMessageType,
    flags: FlagsCompressed,
    session_id: u16,

    chunk_data: Bytes32<'a>,
}

impl<'a> NowFileTransferDataMsg<'a> {
    pub const SUBTYPE: FileTransferMessageType = FileTransferMessageType::Data;

    pub fn new(session_id: u16, flags: FlagsCompressed, chunk_data: &'a [u8]) -> Self {
        Self::new_with_flags_and_chunk_data(session_id, flags, chunk_data)
    }

    pub fn new_with_flags_and_chunk_data(session_id: u16, flags: FlagsCompressed, chunk_data: &'a [u8]) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            session_id,
            chunk_data: Bytes32(chunk_data),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        message::{ChannelName, NowBody, NowVirtualChannel, VirtChannelsCtx},
        packet::NowPacket,
        serialization::{Decode, Encode},
    };

    use super::*;

    fn get_ctx() -> VirtChannelsCtx {
        let mut vchan_ctx = VirtChannelsCtx::new();
        vchan_ctx.insert(0x00, ChannelName::FileTransfer);
        vchan_ctx
    }

    #[rustfmt::skip]
    const NOW_FILE_TRANSFERT_CREATE_REQ_PACKET: [u8; 59] = [
        // vheader
        0x37, 0x00, 0x00, 0x81,
        // file transfer
        0x10, // subtype
        0x00, // flags
        0x00, 0x00, // session id
        0x50, 0x28, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // file size
        0x28, 0x00, 0x54, 0x30, 0x5a, 0x32, 0x58, 0x4d, 0x53, 0x52, 0x5a, 0x2d, 0x55, 0x35, 0x41, 0x32,
        0x34, 0x51, 0x4a, 0x42, 0x47, 0x2d, 0x31, 0x32, 0x64, 0x34, 0x62, 0x65, 0x64, 0x37, 0x38, 0x32,
        0x31, 0x63, 0x2d, 0x35, 0x31, 0x32, 0x2e, 0x70, 0x6e, 0x67, 0x00 // file name
    ];

    #[test]
    fn file_transfer_create_req_decoding() {
        let mut buffer = Vec::new();
        let mut reader = Cursor::new(&NOW_FILE_TRANSFERT_CREATE_REQ_PACKET[..]);
        match NowPacket::read_from(&mut reader, &mut buffer, &get_ctx()) {
            Ok(packet) => match packet.body {
                NowBody::Message(_) => panic!("decoded a now message from a virtual channel packet"),
                NowBody::VirtualChannel(vchan) => {
                    if let NowVirtualChannel::FileTransfer(NowFileTransferMsg::CreateReq(msg)) = vchan {
                        assert_eq!(msg.subtype, FileTransferMessageType::CreateReq);
                        assert_eq!(msg.flags, 0x00);
                        assert_eq!(msg.session_id, 0x0000);
                        assert_eq!(msg.file_size, 75856);
                        assert_eq!(msg.file_name, "T0Z2XMSRZ-U5A24QJBG-12d4bed7821c-512.png");
                    } else {
                        panic!("decoded wrong virtual channel message");
                    }
                }
            },
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode file transfer packet");
            }
        }
    }

    /*#[test]
    fn file_transfer_create_req_encoding() {
        let session_id = 0;
        let file_size = 75856;
        let file_name = "T0Z2XMSRZ-U5A24QJBG-12d4bed7821c-512.png".to_vec();
        let msg = NowFileTransferCreateReqMsg::new(session_id,file_size, NowString65535::from(file_name));
        let channel_id = get_ctx().get_id_by_channel(&ChannelName::FileTransfer).unwrap();
        let packet = NowPacket::from_virt_channel(NowFileTransferMsg::from(msg), channel_id);
        assert_eq!(packet.encode().unwrap(), NOW_FILE_TRANSFERT_CREATE_REQ_PACKET.to_vec());
    }*/
}
