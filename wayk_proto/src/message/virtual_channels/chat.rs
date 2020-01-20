// Chat

use crate::message::common::now_string::NowString65535;
use num_derive::FromPrimitive;

#[derive(Encode, Decode, FromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChatMessageType {
    Sync = 0x00,
    Text = 0x01,
    Read = 0x02,
    Typing = 0x03,
    Name = 0x04,
    Status = 0x05,
    Poke = 0x06,
}

#[derive(Encode, Decode, Debug, Clone)]
#[meta_enum = "ChatMessageType"]
pub enum NowChatMsg {
    Sync(NowChatSyncMsg),
    Text(NowChatTextMsg),
    Read(NowChatReadMsg),
    Typing(NowChatTypingMsg),
    Name(NowChatNameMsg),
    Status(NowChatStatusMsg),
    Poke(NowChatPokeMsg),
}

impl From<NowChatSyncMsg> for NowChatMsg {
    fn from(msg: NowChatSyncMsg) -> Self {
        Self::Sync(msg)
    }
}

impl From<NowChatTextMsg> for NowChatMsg {
    fn from(msg: NowChatTextMsg) -> Self {
        Self::Text(msg)
    }
}

impl From<NowChatReadMsg> for NowChatMsg {
    fn from(msg: NowChatReadMsg) -> Self {
        Self::Read(msg)
    }
}

impl From<NowChatTypingMsg> for NowChatMsg {
    fn from(msg: NowChatTypingMsg) -> Self {
        Self::Typing(msg)
    }
}

impl From<NowChatNameMsg> for NowChatMsg {
    fn from(msg: NowChatNameMsg) -> Self {
        Self::Name(msg)
    }
}

impl From<NowChatStatusMsg> for NowChatMsg {
    fn from(msg: NowChatStatusMsg) -> Self {
        Self::Status(msg)
    }
}

impl From<NowChatPokeMsg> for NowChatMsg {
    fn from(msg: NowChatPokeMsg) -> Self {
        Self::Poke(msg)
    }
}

// subtypes

#[derive(Encode, Decode, FromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChatPresenceStatus {
    Unknown = 0x00,
    Available = 0x01,
    Away = 0x02,
    Idle = 0x03,
    Busy = 0x04,
    DoNotDisturb = 0x05,
    Invisible = 0x06,
    Offline = 0x07,
}

__flags_struct! {
    ChatCapabilitiesFlags: u32 => {
        emoji = EMOJI = 0x0000_0001, // emoji unicode characters supported
        poke = POKE = 0x0000_0002, // poking (attention request) enabled
        read = READ = 0x0000_0004, // read notifications enabled
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowChatSyncMsg {
    subtype: ChatMessageType,
    flags: u8,
    reserved: u16,
    pub timestamp: u32,

    pub capabilities: ChatCapabilitiesFlags,
    pub friendly_name: NowString65535,
    pub presence: ChatPresenceStatus,
    pub status_text: NowString65535,
}

impl NowChatSyncMsg {
    pub const SUBTYPE: ChatMessageType = ChatMessageType::Sync;

    pub fn new(timestamp: u32, capabilities: ChatCapabilitiesFlags, friendly_name: NowString65535) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
            timestamp,
            capabilities,
            friendly_name,
            presence: ChatPresenceStatus::Unknown,
            status_text: NowString65535::new_empty(),
        }
    }

    pub fn presence(self, presence: ChatPresenceStatus) -> Self {
        Self { presence, ..self }
    }

    pub fn status_text(self, status_text: NowString65535) -> Self {
        Self { status_text, ..self }
    }
}

__flags_struct! {
    ChatTextFlags: u8 => {
        snippet = SNIPPET = 0x01, // contains a code snippet (monospace font, no rich formatting).
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowChatTextMsg {
    subtype: ChatMessageType,
    pub flags: ChatTextFlags,
    reserved: u16,
    pub timestamp: u32,

    session_id: u32,
    pub message_id: u32,
    pub text: NowString65535,
}

impl NowChatTextMsg {
    pub const SUBTYPE: ChatMessageType = ChatMessageType::Text;

    pub fn new(timestamp: u32, message_id: u32, text: NowString65535) -> Self {
        Self::new_with_flags(timestamp, message_id, text, ChatTextFlags::new_empty())
    }

    pub fn new_with_flags(timestamp: u32, message_id: u32, text: NowString65535, flags: ChatTextFlags) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            reserved: 0,
            timestamp,
            session_id: 0,
            message_id,
            text,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowChatReadMsg {
    subtype: ChatMessageType,
    flags: u8,
    reserved: u16,
    pub timestamp: u32,
}

impl NowChatReadMsg {
    pub const SUBTYPE: ChatMessageType = ChatMessageType::Read;

    pub fn new(timestamp: u32) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
            timestamp,
        }
    }
}

__flags_struct! {
    ChatTypingFlags: u8 => {
        typing = TYPING = 0x01,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowChatTypingMsg {
    subtype: ChatMessageType,
    flags: u8,
    reserved: u16,
    pub timestamp: u32,

    session_id: u32,
    pub message_id: u32,
}

impl NowChatTypingMsg {
    pub const SUBTYPE: ChatMessageType = ChatMessageType::Typing;

    pub fn new(timestamp: u32, message_id: u32) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
            timestamp,
            session_id: 0,
            message_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowChatNameMsg {
    subtype: ChatMessageType,
    flags: u8,
    reserved: u16,
    pub timestamp: u32,
}

impl NowChatNameMsg {
    const SUBTYPE: ChatMessageType = ChatMessageType::Name;

    pub fn new(timestamp: u32) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
            timestamp,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowChatStatusMsg {
    subtype: ChatMessageType,
    flags: u8,
    reserved: u16,
    pub timestamp: u32,
}

impl NowChatStatusMsg {
    const SUBTYPE: ChatMessageType = ChatMessageType::Status;

    pub fn new(timestamp: u32) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
            timestamp,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowChatPokeMsg {
    subtype: ChatMessageType,
    flags: u8,
    reserved: u16,
    pub timestamp: u32,
}

impl NowChatPokeMsg {
    const SUBTYPE: ChatMessageType = ChatMessageType::Poke;

    pub fn new(timestamp: u32) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
            timestamp,
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
    use std::{io::Cursor, str::FromStr};

    fn get_ctx() -> VirtChannelsCtx {
        let mut vchan_ctx = VirtChannelsCtx::new();
        vchan_ctx.insert(0x03, ChannelName::Chat);
        vchan_ctx
    }

    #[rustfmt::skip]
    const CHAT_SYNC_WITH_HEADER: [u8; 46] = [
        // vheader
        0x2a, 0x00, 0x03, 0x81,
        // chat
        0x00, // subtype
        0x00, // flags
        0x00, 0x00, // reserved
        0xbb, 0xa0, 0x97, 0x5d, // timestamp
        0x05, 0x00, 0x00, 0x00, // capabilities
        // friendly name
        0x17, 0x00,
        0x44, 0x65, 0x76, 0x6f, 0x6c, 0x75, 0x74, 0x69, 0x6f, 0x6e,
        0x73, 0x31, 0x32, 0x38, 0x2f, 0x62, 0x63, 0x6f, 0x72, 0x74,
        0x69, 0x65, 0x72, 0x00,
        // -----
        0x00, // presence
        0x00, 0x00, 0x00 // text status
    ];

    #[test]
    fn decode_chat_sync() {
        let mut buffer = Vec::new();
        let mut reader = Cursor::new(&CHAT_SYNC_WITH_HEADER[..]);
        match NowPacket::read_from(&mut reader, &mut buffer, &get_ctx()) {
            Ok(packet) => match packet.body {
                NowBody::Message(_) => panic!("decoded a now message from a virtual channel packet"),
                NowBody::VirtualChannel(vchan) => {
                    if let NowVirtualChannel::Chat(NowChatMsg::Sync(msg)) = vchan {
                        assert_eq!(msg.subtype, ChatMessageType::Sync);
                        assert_eq!(msg.flags, 0x00);
                        assert_eq!(msg.timestamp, 0x5d97a0bb);
                        assert_eq!(
                            msg.capabilities,
                            ChatCapabilitiesFlags::new_empty().set_emoji().set_read()
                        );
                        assert_eq!(msg.friendly_name, "Devolutions128/bcortier");
                        assert_eq!(msg.presence, ChatPresenceStatus::Unknown);
                        assert_eq!(msg.status_text, "");
                    } else {
                        panic!("decoded wrong virtual channel message");
                    }
                }
            },
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode chat sync packet");
            }
        }
    }

    #[test]
    fn encode_chat_sync() {
        let sync_msg = NowChatSyncMsg::new(
            0x5d97a0bb,
            ChatCapabilitiesFlags::new_empty().set_emoji().set_read(),
            NowString65535::from_str("Devolutions128/bcortier").unwrap(),
        );
        let chat_sync = NowPacket::from_virt_channel(
            NowChatMsg::from(sync_msg),
            get_ctx().get_id_by_channel(&ChannelName::Chat).unwrap(),
        );
        assert_eq!(chat_sync.encode().unwrap(), CHAT_SYNC_WITH_HEADER.to_vec());
    }

    #[rustfmt::skip]
    const TEXT_MSG: [u8; 46] = [
        0x01, // subtype
        0x00, // flags
        0x00, 0x00, // reserved
        0xd1, 0xa0, 0x97, 0x5d, // timestamp
        0x00, 0x00, 0x00, 0x00, // session_id
        0x01, 0x00, 0x00, 0x00, // message_id
        // text
        0x1b, 0x00,
        0xe3, 0x83, 0xa6, 0xe3, 0x83, 0x8b, 0xe3, 0x82, 0xb3, 0xe3,
        0x83, 0xbc, 0xe3, 0x83, 0x89, 0xe3, 0x81, 0xaf, 0xe3, 0x81,
        0xa9, 0xe3, 0x81, 0x86, 0xef, 0xbc, 0x9f, 0x00
    ];

    #[test]
    fn decode_chat_text() {
        let msg = NowChatTextMsg::decode(&TEXT_MSG).unwrap();
        assert_eq!(msg.subtype, ChatMessageType::Text);
        assert_eq!(msg.flags, ChatTextFlags::new_empty());
        assert_eq!(msg.timestamp, 0x5d97a0d1);
        assert_eq!(msg.session_id, 0);
        assert_eq!(msg.message_id, 1);
        assert_eq!(msg.text.as_str(), "ユニコードはどう？");
    }

    #[test]
    fn encode_chat_text() {
        let msg = NowChatTextMsg::new(0x5d97a0d1, 1, NowString65535::from_str("ユニコードはどう？").unwrap());
        assert_eq!(msg.encode().unwrap(), TEXT_MSG.to_vec());
    }
}
