pub mod common;
pub mod connection_sequence;
pub mod now_messages;
pub mod status;
pub mod virtual_channels;

// re-export
pub use common::*;
pub use connection_sequence::*;
pub use now_messages::*;
pub use status::*;
pub use virtual_channels::*;

use crate::error::*;
use crate::io::{Cursor, NoStdWrite};
use crate::serialization::{Decode, Encode};
use alloc::collections::BTreeMap;

// == MESSAGE TYPE == //

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy, Eq)]
pub enum MessageType {
    #[value = 0x00]
    Status,
    #[value = 0x01]
    Handshake,
    #[value = 0x02]
    Negotiate,
    #[value = 0x03]
    Authenticate,
    #[value = 0x04]
    Associate,
    #[value = 0x05]
    Capabilities,
    #[value = 0x06]
    Channel,
    #[value = 0x07]
    Activate,
    #[value = 0x08]
    Terminate,
    #[value = 0x41]
    Surface,
    #[value = 0x42]
    Update,
    #[value = 0x43]
    Input,
    #[value = 0x44]
    Mouse,
    #[value = 0x45]
    Network,
    #[value = 0x46]
    Access,
    #[value = 0x47]
    Desktop,
    #[value = 0x48]
    System,
    #[value = 0x49]
    Session,
    #[value = 0x50]
    Sharing,
    #[fallback]
    Other(u8),
}

// == VIRTUAL CHANNELS CONTEXT ==

#[derive(Debug, Clone)]
pub struct VirtChannelsCtx {
    entries: BTreeMap<u8, ChannelName>,
}

impl Default for VirtChannelsCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtChannelsCtx {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
        }
    }

    pub fn insert(&mut self, id: u8, name: ChannelName) -> Option<ChannelName> {
        self.entries.insert(id, name)
    }

    pub fn get_channel_by_id(&self, id: u8) -> Option<&ChannelName> {
        self.entries.get(&id)
    }

    pub fn get_id_by_channel(&self, name: &ChannelName) -> Option<u8> {
        self.entries.iter().find(|pair| pair.1 == name).map(|pair| *pair.0)
    }
}

// == BODY TYPE == //

#[derive(Debug, Clone, PartialEq, Copy, Eq)]
pub enum BodyType {
    Message(MessageType),
    VirtualChannel(u8),
}

impl Encode for BodyType {
    fn expected_size() -> crate::serialization::ExpectedSize
    where
        Self: Sized,
    {
        crate::serialization::ExpectedSize::Known(1)
    }

    fn encoded_len(&self) -> usize {
        1
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<()>
    where
        Self: Sized,
    {
        match self {
            Self::Message(v) => v.encode_into(writer),
            Self::VirtualChannel(v) => v.encode_into(writer),
        }
    }
}

impl From<MessageType> for BodyType {
    fn from(msg_type: MessageType) -> Self {
        Self::Message(msg_type)
    }
}

impl From<u8> for BodyType {
    fn from(id: u8) -> Self {
        Self::VirtualChannel(id)
    }
}

// == NOW BODY == //

#[derive(Debug, Clone)]
pub enum NowBody<'a> {
    Message(NowMessage<'a>),
    VirtualChannel(NowVirtualChannel<'a>),
}

impl Encode for NowBody<'_> {
    fn expected_size() -> crate::serialization::ExpectedSize
    where
        Self: Sized,
    {
        crate::serialization::ExpectedSize::Variable
    }

    fn encoded_len(&self) -> usize {
        match self {
            Self::Message(msg) => msg.encoded_len(),
            Self::VirtualChannel(msg) => msg.encoded_len(),
        }
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<()>
    where
        Self: Sized,
    {
        match self {
            Self::Message(msg) => msg.encode_into(writer),
            Self::VirtualChannel(msg) => msg.encode_into(writer),
        }
    }
}

impl<'a> From<NowMessage<'a>> for NowBody<'a> {
    fn from(msg: NowMessage<'a>) -> Self {
        Self::Message(msg)
    }
}

impl<'a> From<NowVirtualChannel<'a>> for NowBody<'a> {
    fn from(virt_channel: NowVirtualChannel<'a>) -> Self {
        Self::VirtualChannel(virt_channel)
    }
}

// == NOW VIRTUAL CHANNEL == //

#[derive(Debug, Clone, Encode)]
pub struct CustomVirtualChannel<'a> {
    pub name: ChannelName,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone)]
pub enum NowVirtualChannel<'a> {
    Clipboard(NowClipboardMsg<'a>),
    Chat(NowChatMsg<'a>),
    // TODO: Exec(NowExecMsg),
    // TODO: FileTransfer(NowFileTransferMsg),
    // TODO: Tunnel(NowTunnelMsg),
    Custom(CustomVirtualChannel<'a>),
}

impl<'a> Encode for NowVirtualChannel<'a> {
    fn expected_size() -> crate::serialization::ExpectedSize
    where
        Self: Sized,
    {
        crate::serialization::ExpectedSize::Variable
    }

    fn encoded_len(&self) -> usize {
        match self {
            Self::Clipboard(msg) => msg.encoded_len(),
            Self::Chat(msg) => msg.encoded_len(),
            Self::Custom(msg) => msg.encoded_len(),
        }
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<()>
    where
        Self: Sized,
    {
        match self {
            Self::Clipboard(msg) => msg.encode_into(writer),
            Self::Chat(msg) => msg.encode_into(writer),
            Self::Custom(msg) => msg.encode_into(writer),
        }
    }
}

impl<'a> NowVirtualChannel<'a> {
    pub fn decode_from<'dec: 'a>(channel: &ChannelName, cursor: &mut Cursor<'dec>) -> Result<Self> {
        Ok(match channel {
            ChannelName::Clipboard => Self::Clipboard(NowClipboardMsg::decode_from(cursor)?),
            ChannelName::Chat => Self::Chat(NowChatMsg::decode_from(cursor)?),
            _ => Self::Custom(CustomVirtualChannel {
                name: channel.clone(),
                payload: cursor.read_rest()?,
            }),
        })
    }

    pub fn get_name(&self) -> &ChannelName {
        match self {
            NowVirtualChannel::Clipboard(_) => &ChannelName::Clipboard,
            NowVirtualChannel::Chat(_) => &ChannelName::Chat,
            NowVirtualChannel::Custom(msg) => &msg.name,
        }
    }
}

impl<'a> From<NowClipboardMsg<'a>> for NowVirtualChannel<'a> {
    fn from(msg: NowClipboardMsg<'a>) -> Self {
        Self::Clipboard(msg)
    }
}

impl From<NowClipboardCapabilitiesReqMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardCapabilitiesReqMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::CapabilitiesReq(msg))
    }
}

impl From<NowClipboardCapabilitiesRspMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardCapabilitiesRspMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::CapabilitiesRsp(msg))
    }
}

impl From<NowClipboardControlReqMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardControlReqMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::ControlReq(msg))
    }
}

impl From<NowClipboardControlRspMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardControlRspMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::ControlRsp(msg))
    }
}

impl From<NowClipboardSuspendReqMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardSuspendReqMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::SuspendReq(msg))
    }
}

impl From<NowClipboardSuspendRspMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardSuspendRspMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::SuspendRsp(msg))
    }
}

impl From<NowClipboardResumeReqMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardResumeReqMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::ResumeReq(msg))
    }
}

impl From<NowClipboardResumeRspMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardResumeRspMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::ResumeRsp(msg))
    }
}

impl From<NowClipboardFormatListReqMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardFormatListReqMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::FormatListReq(msg))
    }
}

impl From<NowClipboardFormatListRspMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardFormatListRspMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::FormatListRsp(msg))
    }
}

impl From<NowClipboardFormatDataReqMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardFormatDataReqMsg) -> Self {
        Self::Clipboard(NowClipboardMsg::FormatDataReq(msg))
    }
}

impl<'a> From<NowClipboardFormatDataRspMsg<'a>> for NowVirtualChannel<'a> {
    fn from(msg: NowClipboardFormatDataRspMsg<'a>) -> Self {
        Self::Clipboard(NowClipboardMsg::FormatDataRsp(msg))
    }
}

impl From<NowClipboardFormatDataRspMsgOwned> for NowVirtualChannel<'_> {
    fn from(msg: NowClipboardFormatDataRspMsgOwned) -> Self {
        Self::Clipboard(NowClipboardMsg::FormatDataRspOwned(msg))
    }
}

impl<'a> From<NowChatMsg<'a>> for NowVirtualChannel<'a> {
    fn from(msg: NowChatMsg<'a>) -> Self {
        Self::Chat(msg)
    }
}

impl From<NowChatSyncMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowChatSyncMsg) -> Self {
        Self::Chat(NowChatMsg::Sync(msg))
    }
}

impl From<NowChatTextMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowChatTextMsg) -> Self {
        Self::Chat(NowChatMsg::Text(msg))
    }
}

impl From<NowChatReadMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowChatReadMsg) -> Self {
        Self::Chat(NowChatMsg::Read(msg))
    }
}

impl From<NowChatTypingMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowChatTypingMsg) -> Self {
        Self::Chat(NowChatMsg::Typing(msg))
    }
}

impl From<NowChatNameMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowChatNameMsg) -> Self {
        Self::Chat(NowChatMsg::Name(msg))
    }
}

impl From<NowChatStatusMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowChatStatusMsg) -> Self {
        Self::Chat(NowChatMsg::Status(msg))
    }
}

impl From<NowChatPokeMsg> for NowVirtualChannel<'_> {
    fn from(msg: NowChatPokeMsg) -> Self {
        Self::Chat(NowChatMsg::Poke(msg))
    }
}

impl<'a> From<CustomVirtualChannel<'a>> for NowVirtualChannel<'a> {
    fn from(msg: CustomVirtualChannel<'a>) -> Self {
        Self::Custom(msg)
    }
}

// == NOW MESSAGE == //

#[derive(Debug, Clone)]
pub enum NowMessage<'a> {
    Handshake(NowHandshakeMsg),
    Negotiate(NowNegotiateMsg),
    Authenticate(NowAuthenticateMsg<'a>),
    Associate(NowAssociateMsg<'a>),
    Capabilities(NowCapabilitiesMsg<'a>),
    Channel(NowChannelMsg),
    Activate(NowActivateMsg),
    Terminate(NowTerminateMsg),
    Input(NowInputMsg<'a>),
    Surface(NowSurfaceMsg<'a>),
    Update(NowUpdateMsg<'a>),
    System(NowSystemMsg<'a>),
    Sharing(NowSharingMsg<'a>),
    Access(NowAccessMsg<'a>),
    Custom { ty: MessageType, payload: &'a [u8] },
}

impl<'a> Encode for NowMessage<'a> {
    fn expected_size() -> crate::serialization::ExpectedSize
    where
        Self: Sized,
    {
        crate::serialization::ExpectedSize::Variable
    }

    fn encoded_len(&self) -> usize {
        match self {
            NowMessage::Handshake(m) => m.encoded_len(),
            NowMessage::Negotiate(m) => m.encoded_len(),
            NowMessage::Authenticate(m) => m.encoded_len(),
            NowMessage::Associate(m) => m.encoded_len(),
            NowMessage::Capabilities(m) => m.encoded_len(),
            NowMessage::Channel(m) => m.encoded_len(),
            NowMessage::Activate(m) => m.encoded_len(),
            NowMessage::Terminate(m) => m.encoded_len(),
            NowMessage::Input(m) => m.encoded_len(),
            NowMessage::Surface(m) => m.encoded_len(),
            NowMessage::Update(m) => m.encoded_len(),
            NowMessage::System(m) => m.encoded_len(),
            NowMessage::Sharing(m) => m.encoded_len(),
            NowMessage::Access(m) => m.encoded_len(),
            NowMessage::Custom { payload, .. } => payload.len(),
        }
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<()> {
        match self {
            NowMessage::Handshake(m) => m.encode_into(writer),
            NowMessage::Negotiate(m) => m.encode_into(writer),
            NowMessage::Authenticate(m) => m.encode_into(writer),
            NowMessage::Associate(m) => m.encode_into(writer),
            NowMessage::Capabilities(m) => m.encode_into(writer),
            NowMessage::Channel(m) => m.encode_into(writer),
            NowMessage::Activate(m) => m.encode_into(writer),
            NowMessage::Terminate(m) => m.encode_into(writer),
            NowMessage::Input(m) => m.encode_into(writer),
            NowMessage::Surface(m) => m.encode_into(writer),
            NowMessage::Update(m) => m.encode_into(writer),
            NowMessage::System(m) => m.encode_into(writer),
            NowMessage::Sharing(m) => m.encode_into(writer),
            NowMessage::Access(m) => m.encode_into(writer),
            NowMessage::Custom { payload, .. } => {
                writer.write_all(payload)?;
                Ok(())
            }
        }
    }
}

impl<'a> NowMessage<'a> {
    pub fn decode_from<'dec: 'a>(msg_type: MessageType, cursor: &mut Cursor<'dec>) -> Result<Self> {
        Ok(match msg_type {
            MessageType::Handshake => Self::Handshake(NowHandshakeMsg::decode_from(cursor)?),
            MessageType::Negotiate => Self::Negotiate(NowNegotiateMsg::decode_from(cursor)?),
            MessageType::Authenticate => Self::Authenticate(NowAuthenticateMsg::decode_from(cursor)?),
            MessageType::Associate => Self::Associate(NowAssociateMsg::decode_from(cursor)?),
            MessageType::Capabilities => Self::Capabilities(NowCapabilitiesMsg::decode_from(cursor)?),
            MessageType::Channel => Self::Channel(NowChannelMsg::decode_from(cursor)?),
            MessageType::Activate => Self::Activate(NowActivateMsg::decode_from(cursor)?),
            MessageType::Terminate => Self::Terminate(NowTerminateMsg::decode_from(cursor)?),
            MessageType::Surface => Self::Surface(NowSurfaceMsg::decode_from(cursor)?),
            MessageType::Update => Self::Update(NowUpdateMsg::decode_from(cursor)?),
            MessageType::System => Self::System(NowSystemMsg::decode_from(cursor)?),
            MessageType::Input => Self::Input(NowInputMsg::decode_from(cursor)?),
            MessageType::Sharing => Self::Sharing(NowSharingMsg::decode_from(cursor)?),
            MessageType::Access => Self::Access(NowAccessMsg::decode_from(cursor)?),
            _ => {
                let payload = cursor.read_rest()?;
                Self::Custom { ty: msg_type, payload }
            }
        })
    }

    pub fn get_type(&self) -> MessageType {
        match self {
            NowMessage::Handshake(_) => MessageType::Handshake,
            NowMessage::Negotiate(_) => MessageType::Negotiate,
            NowMessage::Authenticate(_) => MessageType::Authenticate,
            NowMessage::Associate(_) => MessageType::Associate,
            NowMessage::Capabilities(_) => MessageType::Capabilities,
            NowMessage::Channel(_) => MessageType::Channel,
            NowMessage::Activate(_) => MessageType::Activate,
            NowMessage::Terminate(_) => MessageType::Terminate,
            NowMessage::Input(_) => MessageType::Input,
            NowMessage::Surface(_) => MessageType::Surface,
            NowMessage::Update(_) => MessageType::Update,
            NowMessage::System(_) => MessageType::System,
            NowMessage::Sharing(_) => MessageType::Sharing,
            NowMessage::Access(_) => MessageType::Sharing,
            NowMessage::Custom { ty, .. } => *ty,
        }
    }
}

impl From<NowHandshakeMsg> for NowMessage<'_> {
    fn from(msg: NowHandshakeMsg) -> Self {
        Self::Handshake(msg)
    }
}

impl From<NowNegotiateMsg> for NowMessage<'_> {
    fn from(msg: NowNegotiateMsg) -> Self {
        Self::Negotiate(msg)
    }
}

impl<'a> From<NowAuthenticateMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowAuthenticateMsg<'a>) -> Self {
        Self::Authenticate(msg)
    }
}

impl<'a> From<NowAssociateMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowAssociateMsg<'a>) -> Self {
        Self::Associate(msg)
    }
}

impl<'a> From<NowCapabilitiesMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowCapabilitiesMsg<'a>) -> Self {
        Self::Capabilities(msg)
    }
}

impl From<NowChannelMsg> for NowMessage<'_> {
    fn from(msg: NowChannelMsg) -> Self {
        Self::Channel(msg)
    }
}

impl From<NowActivateMsg> for NowMessage<'_> {
    fn from(msg: NowActivateMsg) -> Self {
        Self::Activate(msg)
    }
}

impl From<NowTerminateMsg> for NowMessage<'_> {
    fn from(msg: NowTerminateMsg) -> Self {
        Self::Terminate(msg)
    }
}

impl<'a> From<NowInputMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowInputMsg<'a>) -> Self {
        Self::Input(msg)
    }
}

impl<'a> From<NowSurfaceMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowSurfaceMsg<'a>) -> Self {
        Self::Surface(msg)
    }
}

impl<'a> From<NowUpdateMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowUpdateMsg<'a>) -> Self {
        Self::Update(msg)
    }
}

impl<'a> From<NowSystemMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowSystemMsg<'a>) -> Self {
        Self::System(msg)
    }
}

impl<'a> From<NowSharingMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowSharingMsg<'a>) -> Self {
        Self::Sharing(msg)
    }
}

impl<'a> From<NowAccessMsg<'a>> for NowMessage<'a> {
    fn from(msg: NowAccessMsg<'a>) -> Self {
        Self::Access(msg)
    }
}
