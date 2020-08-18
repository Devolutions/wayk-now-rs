use crate::{
    error::{ProtoErrorKind, ProtoErrorResultExt, Result},
    header::{AbstractNowHeader, NowHeader, NowLongHeader},
    message::{BodyType, MessageType, NowBody, NowMessage, NowVirtualChannel, VirtChannelsCtx},
    serialization::{Decode, Encode},
};
use std::{
    io::{Cursor, Read, Write},
    marker::PhantomData,
};

/// A raw now packet.
///
/// Decodes only the header, the payload
/// have to be decoded manually.
/// Doesn't provides encoding.
/// See [`NowPacket`](struct.NowPacket.html).
pub struct NowRawPacket<'a> {
    pub header: NowHeader,
    pub payload: &'a [u8],
}

sa::assert_impl_all!(NowRawPacket: Sync, Send);

impl<'dec: 'a, 'a> Decode<'dec> for NowRawPacket<'a> {
    fn decode_from(cursor: &mut Cursor<&'dec [u8]>) -> Result<Self> {
        let header = NowHeader::decode_from(cursor)?;
        let payload = &cursor.get_ref()[cursor.position() as usize..];

        Ok(Self { header, payload })
    }
}

/// A now packet.
///
/// See [`NowRawPacket`](struct.NowRawPacket.html) if you would rather decode by hand.
#[derive(Debug, Clone)]
pub struct NowPacket<'a> {
    pub header: NowHeader,
    pub body: NowBody<'a>,
}

sa::assert_impl_all!(NowPacket: Sync, Send);

impl Encode for NowPacket<'_> {
    fn encoded_len(&self) -> usize {
        self.header.encoded_len() + self.body.encoded_len()
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.header.encode_into(writer)?;
        self.body.encode_into(writer)
    }
}

impl<'a> NowPacket<'a> {
    pub fn from_message<Message: Into<NowMessage<'a>>>(message: Message) -> Self {
        let message = message.into();

        let header = match &message {
            NowMessage::Handshake(msg) => {
                NowHeader::new_with_msg_type(MessageType::Handshake, msg.encoded_len() as u32)
            }
            NowMessage::Negotiate(msg) => {
                NowHeader::new_with_msg_type(MessageType::Negotiate, msg.encoded_len() as u32)
            }
            NowMessage::Authenticate(msg) => {
                NowHeader::new_with_msg_type(MessageType::Authenticate, msg.encoded_len() as u32)
            }
            NowMessage::Associate(msg) => {
                NowHeader::new_with_msg_type(MessageType::Associate, msg.encoded_len() as u32)
            }
            NowMessage::Capabilities(msg) => {
                NowHeader::new_with_msg_type(MessageType::Capabilities, msg.encoded_len() as u32)
            }
            NowMessage::Channel(msg) => NowHeader::new_with_msg_type(MessageType::Channel, msg.encoded_len() as u32),
            NowMessage::Activate(msg) => NowHeader::new_with_msg_type(MessageType::Activate, msg.encoded_len() as u32),
            NowMessage::Terminate(msg) => {
                NowHeader::new_with_msg_type(MessageType::Terminate, msg.encoded_len() as u32)
            }
            NowMessage::Input(msg) => NowHeader::new_with_msg_type(MessageType::Input, msg.encoded_len() as u32),
            NowMessage::Surface(msg) => NowHeader::new_with_msg_type(MessageType::Surface, msg.encoded_len() as u32),
            NowMessage::Update(msg) => NowHeader::new_with_msg_type(MessageType::Update, msg.encoded_len() as u32),
            NowMessage::System(msg) => NowHeader::new_with_msg_type(MessageType::System, msg.encoded_len() as u32),
            NowMessage::Sharing(msg) => NowHeader::new_with_msg_type(MessageType::Sharing, msg.encoded_len() as u32),
            NowMessage::Access(msg) => NowHeader::new_with_msg_type(MessageType::Access, msg.encoded_len() as u32),
        };

        Self {
            header,
            body: NowBody::Message(message),
        }
    }

    pub fn from_virt_channel<Channel: Into<NowVirtualChannel<'a>>>(virt_channel: Channel, channel_id: u8) -> Self {
        let virt_channel = virt_channel.into();
        let header = NowHeader::new_with_virt_channel(channel_id, virt_channel.encoded_len() as u32);

        Self {
            header,
            body: NowBody::VirtualChannel(virt_channel),
        }
    }

    pub fn read_from<'dec: 'a, R: Read>(
        reader: &mut R,
        buffer: &'dec mut Vec<u8>,
        channels_ctx: &VirtChannelsCtx,
    ) -> Result<Self> {
        let header = NowHeader::read_from(reader)?;
        let message_len = header.body_len();

        buffer.clear();
        if buffer.capacity() < message_len {
            buffer.reserve_exact(message_len - buffer.capacity());
        }
        reader.take(message_len as u64).read_to_end(buffer)?;

        Self::decode_from(header, buffer, channels_ctx)
    }

    pub fn decode_from<'dec: 'a>(
        header: NowHeader,
        buffer: &'dec [u8],
        channels_ctx: &VirtChannelsCtx,
    ) -> Result<Self> {
        let mut cursor = Cursor::new(buffer);
        let body = match header.body_type() {
            BodyType::Message(msg_type) => NowBody::Message(NowMessage::decode_from(msg_type, &mut cursor)?),
            BodyType::VirtualChannel(id) => {
                let channel_name = channels_ctx
                    .get_channel_by_id(id)
                    .chain(ProtoErrorKind::Decoding("NowPacket"))
                    .or_desc("channel name not found in channels context")?;
                NowBody::VirtualChannel(NowVirtualChannel::decode_from(channel_name, &mut cursor)?)
            }
        };

        Ok(Self { header, body })
    }
}

impl<'a, Message> From<Message> for NowPacket<'a>
where
    Message: Into<NowMessage<'a>>,
{
    fn from(message: Message) -> Self {
        NowPacket::from_message(message)
    }
}

/// Accumulate bytes to build into packets
#[derive(Debug, Clone)]
pub struct NowPacketAccumulator<'a> {
    buffer: Vec<u8>,
    cursor: usize,
    _pd: PhantomData<&'a ()>,
}

sa::assert_impl_all!(NowPacketAccumulator: Sync, Send);

impl Default for NowPacketAccumulator<'_> {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
            _pd: PhantomData,
        }
    }
}

impl NowPacketAccumulator<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accumulate(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn purge_old_packets(&mut self) {
        self.buffer.drain(..self.cursor);
        self.cursor = 0;
    }

    pub fn next_packet<'a>(&'a mut self, channels_ctx: &VirtChannelsCtx) -> Option<Result<NowPacket<'a>>> {
        if self.buffer.len() < self.cursor + NowLongHeader::SIZE {
            return None;
        }

        let header = match NowHeader::decode(&self.buffer[self.cursor..self.cursor + NowLongHeader::SIZE]) {
            Ok(header) => header,
            Err(err) => return Some(Err(err)),
        };

        let packet_len = header.body_len() + header.len();
        if self.buffer.len() >= self.cursor + packet_len {
            let header_len = header.len();
            let packet = NowPacket::decode_from(
                header,
                &self.buffer[self.cursor + header_len..self.cursor + packet_len],
                channels_ctx,
            );
            self.cursor += packet_len;
            Some(packet)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{AuthType, NowBody, VirtChannelsCtx};

    #[rustfmt::skip]
    const NEGOTIATE_PACKET: [u8; 11] = [
        // vheader
        0x07, 0x00, // size
        0x02, // subtye
        0x80, // flags

        // negotiate
        0x01, 0x00, 0x00, 0x00, // flags
        0x02, // count available auths
        0x02, // SRP
        0x01, // PFP
    ];

    #[test]
    fn now_packet_decoding_with_accumulator() {
        let chan_ctx = VirtChannelsCtx::new();

        let mut acc = NowPacketAccumulator::new();
        acc.accumulate(&NEGOTIATE_PACKET[..6]);
        assert_eq!(acc.buffer.len(), 6);
        assert!(acc.next_packet(&chan_ctx).is_none());
        acc.accumulate(&NEGOTIATE_PACKET[6..]);
        assert_eq!(acc.buffer.len(), 11);
        assert_eq!(acc.cursor, 0);

        let packet_result = acc.next_packet(&chan_ctx).unwrap();
        match packet_result {
            Ok(packet) => match packet.body {
                NowBody::Message(msg) => match msg {
                    NowMessage::Negotiate(msg) => {
                        assert!(msg.flags.srp_extended());
                        assert_eq!(msg.auth_list.len(), 2);
                        assert_eq!(msg.auth_list[0], AuthType::SRP);
                        assert_eq!(msg.auth_list[1], AuthType::PFP);
                    }
                    _ => panic!("decoded wrong now message from negotiate response packet"),
                },
                NowBody::VirtualChannel(_) => panic!("decoded a virtual channel message from a negotiate packet"),
            },
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode negotiate packet");
            }
        }

        assert!(acc.next_packet(&chan_ctx).is_none());
        assert_eq!(acc.cursor, 11);
        acc.purge_old_packets();
        assert_eq!(acc.cursor, 0);
        assert_eq!(acc.buffer.len(), 0);
    }
}
