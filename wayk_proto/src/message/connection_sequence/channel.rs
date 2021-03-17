use alloc::borrow::{Borrow, Cow};
use core::str::FromStr;
use std::io::{Cursor, Write};
use wayk_proto::container::Vec8;
use wayk_proto::error::Result;
use wayk_proto::message::NowString64;
use wayk_proto::serialization::{Decode, Encode};

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum ChannelMessageType {
    #[value = 0x01]
    ChannelListRequest,
    #[value = 0x02]
    ChannelListResponse,
    #[value = 0x03]
    ChannelOpenRequest,
    #[value = 0x04]
    ChannelOpenResponse,
    #[value = 0x05]
    ChannelCloseRequest,
    #[value = 0x06]
    ChannelCloseResponse,
    #[value = 0x07]
    ChannelStartRequest,
    #[value = 0x08]
    ChannelStartResponse,
    #[value = 0x09]
    ChannelStopRequest,
    #[value = 0x0a]
    ChannelStopResponse,
    #[fallback]
    Other(u8),
}

__flags_struct! {
    ChannelDefFlags: u32 => {
        dynamic = DYNAMIC = 0x0000_0001,
        multiple = MULTIPLE = 0x0000_0002,
        stopped = STOPPED = 0x0000_0004,
        server = SERVER = 0x0001_0000,
        is_async = ASYNC = 0x0002_0000,
        irp = IRP = 0x0004_0000,
        local = LOCAL = 0x0008_0000,
        proxy = PROXY = 0x0010_0000,
        status = STATUS = 0x8000_0000,
        status_success = STATUS_SUCCESS = 0x8000_0000,
        status_failure = STATUS_FAILURE = 0x8000_0001,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowChannelDef {
    pub flags: ChannelDefFlags,
    pub name: ChannelName,
}

impl NowChannelDef {
    pub fn new(name: ChannelName) -> Self {
        Self::new_with_flags(name, ChannelDefFlags::new_empty())
    }

    pub fn new_with_flags(name: ChannelName, flags: ChannelDefFlags) -> Self {
        Self { flags, name }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum ChannelName {
    Unknown(Cow<'static, str>),
    Clipboard,
    FileTransfer,
    Exec,
    Chat,
    Tunnel,
}

impl Encode for ChannelName {
    fn encoded_len(&self) -> usize {
        let name = match self {
            ChannelName::Unknown(name) => name.borrow(),
            ChannelName::Clipboard => Self::CLIPBOARD_STR,
            ChannelName::FileTransfer => Self::FILE_TRANSFER_STR,
            ChannelName::Exec => Self::EXEC_STR,
            ChannelName::Chat => Self::CHAT_STR,
            ChannelName::Tunnel => Self::TUNNEL_STR,
        };
        name.len() + 2
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        let name = NowString64::from_str(self.as_str())?;
        name.encode_into(writer)?;
        Ok(())
    }
}

impl<'dec: 'a, 'a> Decode<'dec> for ChannelName {
    fn decode_from(cursor: &mut Cursor<&'dec [u8]>) -> Result<Self> {
        let name = NowString64::decode_from(cursor)?;
        match name.as_str() {
            Self::CLIPBOARD_STR => Ok(Self::Clipboard),
            Self::FILE_TRANSFER_STR => Ok(Self::FileTransfer),
            Self::EXEC_STR => Ok(Self::Exec),
            Self::CHAT_STR => Ok(Self::Chat),
            Self::TUNNEL_STR => Ok(Self::Tunnel),
            _ => Ok(Self::Unknown(name.into())),
        }
    }
}

impl ChannelName {
    pub const CLIPBOARD_STR: &'static str = "NowClipboard";
    pub const FILE_TRANSFER_STR: &'static str = "NowFileTransfer";
    pub const EXEC_STR: &'static str = "NowExec";
    pub const CHAT_STR: &'static str = "NowChat";
    pub const TUNNEL_STR: &'static str = "NowTunnel";

    pub fn as_str(&self) -> &str {
        match self {
            Self::Unknown(name) => name,
            Self::Clipboard => Self::CLIPBOARD_STR,
            Self::FileTransfer => Self::FILE_TRANSFER_STR,
            Self::Exec => Self::EXEC_STR,
            Self::Chat => Self::CHAT_STR,
            Self::Tunnel => Self::TUNNEL_STR,
        }
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct NowChannelMsg {
    pub subtype: ChannelMessageType,
    flags: u8,
    pub channel_list: Vec8<NowChannelDef>,
}

impl NowChannelMsg {
    pub fn new(subtype: ChannelMessageType, channel_list: Vec<NowChannelDef>) -> Self {
        Self {
            subtype,
            flags: 0x0,
            channel_list: Vec8(channel_list),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::VirtChannelsCtx;
    use crate::packet::NowPacket;

    const CHANNEL_LIST_REQUEST_PACKET: [u8; 72] = [
        0x44, 0x00, 0x06, 0x80, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x4e, 0x6f, 0x77, 0x43, 0x6c, 0x69,
        0x70, 0x62, 0x6f, 0x61, 0x72, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x4e, 0x6f, 0x77, 0x46, 0x69, 0x6c,
        0x65, 0x54, 0x72, 0x61, 0x6e, 0x73, 0x66, 0x65, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x4e, 0x6f, 0x77,
        0x45, 0x78, 0x65, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x4e, 0x6f, 0x77, 0x43, 0x68, 0x61, 0x74, 0x00,
    ];

    #[test]
    fn full_decode() {
        let mut buffer = Vec::new();
        let mut reader = Cursor::new(&CHANNEL_LIST_REQUEST_PACKET[..]);
        match NowPacket::read_from(&mut reader, &mut buffer, &VirtChannelsCtx::new()) {
            Ok(_) => {}
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode now channels packet");
            }
        }
    }

    #[test]
    fn full_encode() {
        let channels_list = vec![
            NowChannelDef {
                flags: ChannelDefFlags::new_empty(),
                name: ChannelName::Clipboard,
            },
            NowChannelDef {
                flags: ChannelDefFlags::new_empty(),
                name: ChannelName::FileTransfer,
            },
            NowChannelDef {
                flags: ChannelDefFlags::new_empty(),
                name: ChannelName::Exec,
            },
            NowChannelDef {
                flags: ChannelDefFlags::new_empty(),
                name: ChannelName::Chat,
            },
        ];
        let packet = NowPacket::from_message(NowChannelMsg::new(
            ChannelMessageType::ChannelListRequest,
            channels_list,
        ));
        assert_eq!(packet.encode().unwrap(), CHANNEL_LIST_REQUEST_PACKET.to_vec());
    }

    const CLIPBOARD_CHANNEL_DEF: [u8; 18] = [
        0x00, 0x00, 0x00, 0x00, 0x0c, 0x4e, 0x6f, 0x77, 0x43, 0x6c, 0x69, 0x70, 0x62, 0x6f, 0x61, 0x72, 0x64, 0x00,
    ];

    #[test]
    fn clipboard_channel_def_encode() {
        let clipboard_channel_def = NowChannelDef {
            flags: ChannelDefFlags::new_empty(),
            name: ChannelName::Clipboard,
        };
        assert_eq!(clipboard_channel_def.encode().unwrap(), CLIPBOARD_CHANNEL_DEF.to_vec())
    }

    #[test]
    fn clipboard_channel_def_decode() {
        let clipboard_channel_def = NowChannelDef::decode(&CLIPBOARD_CHANNEL_DEF).unwrap();
        assert_eq!(clipboard_channel_def.flags, ChannelDefFlags::new_empty());
        assert_eq!(ChannelName::Clipboard, clipboard_channel_def.name)
    }

    const FILE_TRANSFER_CHANNEL_DEF: [u8; 21] = [
        0x00, 0x00, 0x00, 0x00, 0x0f, 0x4e, 0x6f, 0x77, 0x46, 0x69, 0x6c, 0x65, 0x54, 0x72, 0x61, 0x6e, 0x73, 0x66,
        0x65, 0x72, 0x00,
    ];

    #[test]
    fn file_transfer_channel_def_encode() {
        let file_transfer_channel_def = NowChannelDef {
            flags: ChannelDefFlags::new_empty(),
            name: ChannelName::FileTransfer,
        };
        assert_eq!(
            file_transfer_channel_def.encode().unwrap(),
            FILE_TRANSFER_CHANNEL_DEF.to_vec()
        )
    }

    #[test]
    fn file_transfer_channel_def_decode() {
        let file_transfer_channel_def = NowChannelDef::decode(&FILE_TRANSFER_CHANNEL_DEF).unwrap();
        assert_eq!(file_transfer_channel_def.flags, ChannelDefFlags::new_empty());
        assert_eq!(ChannelName::FileTransfer, file_transfer_channel_def.name);
    }

    const EXEC_CHANNEL_DEF: [u8; 13] = [
        0x00, 0x00, 0x00, 0x00, 0x07, 0x4e, 0x6f, 0x77, 0x45, 0x78, 0x65, 0x63, 0x00,
    ];

    #[test]
    fn exec_channel_def_encode() {
        let exec_channel_def = NowChannelDef {
            flags: ChannelDefFlags::new_empty(),
            name: ChannelName::Exec,
        };
        assert_eq!(exec_channel_def.encode().unwrap(), EXEC_CHANNEL_DEF.to_vec())
    }

    #[test]
    fn exec_channel_def_decode() {
        let exec_channel_def = NowChannelDef::decode(&EXEC_CHANNEL_DEF).unwrap();
        assert_eq!(exec_channel_def.flags, ChannelDefFlags::new_empty());
        assert_eq!(ChannelName::Exec, exec_channel_def.name);
    }

    const CHAT_CHANNEL_DEF: [u8; 13] = [
        0x00, 0x00, 0x00, 0x00, 0x07, 0x4e, 0x6f, 0x77, 0x43, 0x68, 0x61, 0x74, 0x00,
    ];

    #[test]
    fn chat_channel_def_encode() {
        let chat_channel_def = NowChannelDef {
            flags: ChannelDefFlags::new_empty(),
            name: ChannelName::Chat,
        };
        assert_eq!(chat_channel_def.encode().unwrap(), CHAT_CHANNEL_DEF.to_vec())
    }

    #[test]
    fn chat_channel_def_decode() {
        let chat_channel_def = NowChannelDef::decode(&CHAT_CHANNEL_DEF).unwrap();
        assert_eq!(chat_channel_def.flags, ChannelDefFlags::new_empty());
        assert_eq!(ChannelName::Chat, chat_channel_def.name);
    }

    const UNKNOWN_CHANNEL_DEF: [u8; 15] = [
        0x00, 0x00, 0x00, 0x00, 0x9, 0x53, 0x6f, 0x6d, 0x65, 0x74, 0x68, 0x69, 0x6e, 0x67, 0x0,
    ];

    #[test]
    fn unknown_channel_def_encode() {
        let unknown_channel_def = NowChannelDef {
            flags: ChannelDefFlags::new_empty(),
            name: ChannelName::Unknown("Something".into()),
        };
        assert_eq!(unknown_channel_def.encode().unwrap(), UNKNOWN_CHANNEL_DEF.to_vec())
    }

    #[test]
    fn unknown_channel_def_decode() {
        let unknown_channel_def = NowChannelDef::decode(&UNKNOWN_CHANNEL_DEF).unwrap();
        assert_eq!(unknown_channel_def.flags, ChannelDefFlags::new_empty());
        assert_eq!(unknown_channel_def.name.as_str(), "Something");
    }
}
