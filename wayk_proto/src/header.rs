use crate::{
    error::*,
    message::{BodyType, MessageType},
    serialization::{Decode, Encode},
};
use std::io::{Cursor, Read, Write};

const HEADER_VIRTUAL_CHANNEL_FLAG: u8 = 0x01;

#[allow(clippy::len_without_is_empty)] // it doesn't make sense in our case
pub trait AbstractNowHeader {
    fn len(&self) -> usize;
    fn is_short(&self) -> bool;
    fn flags(&self) -> u8;
    fn body_type(&self) -> BodyType;
    fn body_len(&self) -> usize;
    fn packet_len(&self) -> usize;
}

#[derive(Debug, Clone)]
pub enum NowHeader {
    Short(NowShortHeader),
    Long(NowLongHeader),
}

impl Decode<'_> for NowHeader {
    fn decode_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        Self::read_from(cursor)
    }
}

impl Encode for NowHeader {
    fn encoded_len(&self) -> usize {
        match self {
            NowHeader::Short(_) => NowShortHeader::SIZE,
            NowHeader::Long(_) => NowLongHeader::SIZE,
        }
    }

    fn encode_into<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            NowHeader::Short(header) => header.encode_into(writer),
            NowHeader::Long(header) => header.encode_into(writer),
        }
    }
}

impl NowHeader {
    pub fn new(body_type: BodyType, body_len: u32) -> Self {
        if body_len > u32::from(u16::max_value()) {
            Self::Long(NowLongHeader::new(body_type, body_len))
        } else {
            Self::Short(NowShortHeader::new(body_type, body_len as u16))
        }
    }

    pub fn new_with_msg_type(msg_type: MessageType, body_len: u32) -> Self {
        Self::new(BodyType::Message(msg_type), body_len)
    }

    pub fn new_with_virt_channel(channel_id: u8, body_len: u32) -> Self {
        Self::new(BodyType::VirtualChannel(channel_id), body_len)
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (bytes, short_bit) = {
            let mut buffer = vec![0u8; 4];
            reader
                .read_exact(&mut buffer)
                .map_err(ProtoError::from)
                .chain(ProtoErrorKind::Decoding(stringify!(NowHeader)))
                .or_desc("couldn't read short bit (no enough bytes provided")?;

            let is_short = buffer[3] > 7;

            if !is_short {
                buffer.append(&mut vec![0u8; 2]);
                reader
                    .read_exact(&mut buffer[4..6])
                    .map_err(ProtoError::from)
                    .chain(ProtoErrorKind::Decoding(stringify!(NowHeader)))
                    .or_desc("not enough bytes provided to parse long header")?;
            };

            (buffer, is_short)
        };

        let mut cursor = Cursor::new(&bytes[..]);
        if short_bit {
            Ok(NowHeader::Short(NowShortHeader::decode_from(&mut cursor)?))
        } else {
            Ok(NowHeader::Long(NowLongHeader::decode_from(&mut cursor)?))
        }
    }

    pub fn borrow_short(&self) -> Option<&NowShortHeader> {
        match self {
            NowHeader::Short(header) => Some(header),
            NowHeader::Long(_) => None,
        }
    }

    pub fn into_short(self) -> Option<NowShortHeader> {
        match self {
            NowHeader::Short(header) => Some(header),
            NowHeader::Long(_) => None,
        }
    }

    pub fn borrow_long(&self) -> Option<&NowLongHeader> {
        match self {
            NowHeader::Short(_) => None,
            NowHeader::Long(header) => Some(header),
        }
    }

    pub fn into_long(self) -> Option<NowLongHeader> {
        match self {
            NowHeader::Short(_) => None,
            NowHeader::Long(header) => Some(header),
        }
    }

    pub fn borrow_abstract(&self) -> Box<&dyn AbstractNowHeader> {
        match self {
            NowHeader::Short(header) => Box::new(header),
            NowHeader::Long(header) => Box::new(header),
        }
    }

    pub fn into_abstract(self) -> Box<dyn AbstractNowHeader> {
        match self {
            NowHeader::Short(header) => Box::new(header),
            NowHeader::Long(header) => Box::new(header),
        }
    }
}

impl AbstractNowHeader for NowHeader {
    fn len(&self) -> usize {
        match self {
            NowHeader::Short(hdr) => hdr.len(),
            NowHeader::Long(hdr) => hdr.len(),
        }
    }

    fn is_short(&self) -> bool {
        match self {
            NowHeader::Short(hdr) => hdr.is_short(),
            NowHeader::Long(hdr) => hdr.is_short(),
        }
    }

    fn flags(&self) -> u8 {
        match self {
            NowHeader::Short(hdr) => hdr.flags(),
            NowHeader::Long(hdr) => hdr.flags(),
        }
    }

    fn body_type(&self) -> BodyType {
        match self {
            NowHeader::Short(hdr) => hdr.body_type(),
            NowHeader::Long(hdr) => hdr.body_type(),
        }
    }

    fn body_len(&self) -> usize {
        match self {
            NowHeader::Short(hdr) => hdr.body_len(),
            NowHeader::Long(hdr) => hdr.body_len(),
        }
    }

    fn packet_len(&self) -> usize {
        match self {
            NowHeader::Short(hdr) => hdr.packet_len(),
            NowHeader::Long(hdr) => hdr.packet_len(),
        }
    }
}

#[derive(Encode, Debug, PartialEq, Clone)]
pub struct NowShortHeader {
    body_len: u16,
    body_type: BodyType,
    flags: u8,
}

impl Decode<'_> for NowShortHeader {
    fn decode_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        let body_len = u16::decode_from(cursor)?;
        let body_type_raw = u8::decode_from(cursor)?;
        let flags = u8::decode_from(cursor)?;
        let body_type = if flags & HEADER_VIRTUAL_CHANNEL_FLAG != 0 {
            BodyType::VirtualChannel(body_type_raw)
        } else {
            BodyType::Message(MessageType::decode(&[body_type_raw])?)
        };

        Ok(Self {
            body_len,
            body_type,
            flags,
        })
    }
}

impl NowShortHeader {
    pub const SIZE: usize = 4;

    pub fn new(body_type: BodyType, body_len: u16) -> Self {
        let flags = 0x80 // short bit flag
            | if let BodyType::VirtualChannel { .. } = body_type { HEADER_VIRTUAL_CHANNEL_FLAG } else { 0x00 };

        Self {
            flags,
            body_type,
            body_len,
        }
    }

    pub fn new_with_msg_type(message_type: MessageType, body_len: u16) -> Self {
        Self::new(BodyType::Message(message_type), body_len)
    }

    pub fn new_with_virt_channel(virtual_channel_id: u8, body_len: u16) -> Self {
        Self::new(BodyType::VirtualChannel(virtual_channel_id), body_len)
    }
}

impl AbstractNowHeader for NowShortHeader {
    fn len(&self) -> usize {
        NowShortHeader::SIZE
    }

    fn is_short(&self) -> bool {
        true
    }

    fn flags(&self) -> u8 {
        self.flags & 0b0111_1111 // unset the short bit flag
    }

    fn body_type(&self) -> BodyType {
        self.body_type
    }

    fn body_len(&self) -> usize {
        self.body_len as usize
    }

    fn packet_len(&self) -> usize {
        self.body_len as usize + Self::SIZE
    }
}

#[derive(Debug, Encode, PartialEq, Clone)]
pub struct NowLongHeader {
    body_len: u32,
    flags: u8,
    body_type: BodyType,
}

impl Decode<'_> for NowLongHeader {
    fn decode_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        let body_len = u32::decode_from(cursor)?;
        let flags = u8::decode_from(cursor)?;

        let body_type = if flags & HEADER_VIRTUAL_CHANNEL_FLAG != 0 {
            BodyType::VirtualChannel(u8::decode_from(cursor)?)
        } else {
            BodyType::Message(MessageType::decode_from(cursor)?)
        };

        Ok(Self {
            body_len,
            flags,
            body_type,
        })
    }
}

impl NowLongHeader {
    pub const SIZE: usize = 6;

    pub fn new(body_type: BodyType, body_size: u32) -> Self {
        Self {
            body_len: body_size & 0b0111_1111_1111_1111u32, // unset short bit flag
            flags: if let BodyType::VirtualChannel { .. } = body_type {
                HEADER_VIRTUAL_CHANNEL_FLAG
            } else {
                0x00
            },
            body_type,
        }
    }

    pub fn new_with_msg_type(message_type: MessageType, body_len: u32) -> Self {
        Self::new(BodyType::Message(message_type), body_len)
    }

    pub fn new_with_virt_channel(virtual_channel_id: u8, body_len: u32) -> Self {
        Self::new(BodyType::VirtualChannel(virtual_channel_id), body_len)
    }
}

impl AbstractNowHeader for NowLongHeader {
    fn len(&self) -> usize {
        NowLongHeader::SIZE
    }

    fn is_short(&self) -> bool {
        false
    }

    fn flags(&self) -> u8 {
        self.flags
    }

    fn body_type(&self) -> BodyType {
        self.body_type
    }

    fn body_len(&self) -> usize {
        self.body_len as usize
    }

    fn packet_len(&self) -> usize {
        self.body_len as usize + Self::SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    #[rustfmt::skip]
    const SHORT_HEADER_MSG: [u8; 4] = [
        // vheader
        0x28, 0x00, // msg size
        0x01, // msg type
        0x80, // msg flags
    ];

    #[test]
    fn short_header_decoding() {
        let header = NowHeader::decode(&SHORT_HEADER_MSG).unwrap().into_short().unwrap();
        assert!(header.is_short());
        assert_eq!(header.flags(), 0x00);
        assert_eq!(header.flags, 0x80); // raw flags, test the short bit
        assert_eq!(header.body_type(), BodyType::Message(MessageType::Handshake));
        assert_eq!(header.body_len(), 40);
    }

    #[test]
    fn short_header_decoding_error() {
        let header = NowHeader::decode(&SHORT_HEADER_MSG).unwrap().into_long();
        assert_eq!(header, None); // wasn't a long header
    }

    #[test]
    fn short_header_encoding() {
        let header = NowShortHeader::new_with_msg_type(MessageType::Handshake, 40);
        assert_eq!([0x28, 0x00, 0x01, 0x80], header.encode().unwrap()[0..]);
    }

    #[rustfmt::skip]
    const LONG_HEADER_MSG: [u8; 6] = [
        // header
        0x1d, 0x03, 0x00, 0x00, // msg size
        0x00, // msg flags
        0x42, // msg type
    ];

    #[test]
    fn long_header_decoding() {
        let header = NowHeader::decode(&LONG_HEADER_MSG).unwrap().into_abstract();
        assert!(!header.is_short());
        assert_eq!(header.flags(), 0x00);
        assert_eq!(header.body_type(), BodyType::Message(MessageType::Update));
        assert_eq!(header.body_len(), 797);
    }

    #[test]
    fn long_header_encoding() {
        let header = NowLongHeader::new_with_msg_type(MessageType::Update, 797);
        assert_eq!([0x1d, 0x03, 0x00, 0x00, 0x00, 0x42], header.encode().unwrap()[..]);
    }

    #[rustfmt::skip]
    const VIRTUAL_CHANNEL_HEADER: [u8; 20] = [
        // vheader
        0x10, 0x00, // msg size
        0x01, // channel id
        0x81, // flags

        // file transfer channel message
        0x00, // subtype
        0x00, // flags
        0x00, 0x00, // session id
        0x00, 0x00, 0x00, 0x00, // capabilities
        0x00, 0x20, 0x00, 0x00, // chunk size
        0x00, 0x00, 0x00, 0x00, // flags
    ];

    #[test]
    fn channel_header_decoding() {
        let header = NowHeader::decode(&VIRTUAL_CHANNEL_HEADER).unwrap();
        assert!(header.is_short());
        assert_eq!(header.flags(), HEADER_VIRTUAL_CHANNEL_FLAG);
        assert_eq!(header.body_type(), BodyType::VirtualChannel(0x01));
        assert_eq!(header.body_len(), 16);
    }

    #[test]
    fn channel_header_encoding() {
        let header = NowHeader::new_with_virt_channel(0x01, 16);
        assert_eq!([0x10, 0x00, 0x01, 0x81], header.encode().unwrap()[..]);
    }
}
