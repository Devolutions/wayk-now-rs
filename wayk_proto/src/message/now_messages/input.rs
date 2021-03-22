use crate::io::{Cursor, NoStdWrite};
use alloc::vec::Vec;
use core::mem;
use wayk_proto::container::Vec16;
use wayk_proto::error::*;
use wayk_proto::message::connection_sequence::InputActionCode;
use wayk_proto::serialization::{Decode, Encode};

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum InputMessageType {
    #[value = 0x01]
    Mouse,
    #[value = 0x02]
    Scroll,
    #[value = 0x03]
    Keyboard,
    #[value = 0x04]
    Unicode,
    #[value = 0x05]
    Toggle,
    #[value = 0x06]
    Action,
    #[fallback]
    Other(u8),
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum EventMouseFlags {
    #[value = 0x0]
    None,
    #[value = 0x01]
    ButtonLeft,
    #[value = 0x02]
    ButtonRight,
    #[value = 0x04]
    ButtonMiddle,
    #[value = 0x10]
    ButtonX1,
    #[value = 0x20]
    ButtonX2,
    #[fallback]
    Other(u8),
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct NowInputEventMouse {
    subtype: InputMessageType,
    pub flags: EventMouseFlags,
    pub x: i16,
    pub y: i16,
}

impl NowInputEventMouse {
    pub fn new_with_flags_and_position(flags: EventMouseFlags, x: i16, y: i16) -> Self {
        Self {
            subtype: InputMessageType::Mouse,
            flags,
            x,
            y,
        }
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct NowInputEventScroll {
    subtype: InputMessageType,
    flags: u8,
    pub x: i16,
    pub y: i16,
}

impl NowInputEventScroll {
    pub fn new_with_position(x: i16, y: i16) -> Self {
        Self {
            subtype: InputMessageType::Scroll,
            flags: 0x0,
            x,
            y,
        }
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct NowInputEventKeyboard {
    subtype: InputMessageType,
    pub flags: u8,
    pub code: u16,
}

impl NowInputEventKeyboard {
    pub fn new_with_flags_and_code(flags: u8, code: u16) -> Self {
        Self {
            subtype: InputMessageType::Keyboard,
            flags,
            code,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NowInputEventUnicode {
    subtype: InputMessageType,
    pub code: Vec<u8>,
}

impl<'a> Encode for NowInputEventUnicode {
    fn expected_size() -> crate::serialization::ExpectedSize
    where
        Self: Sized,
    {
        crate::serialization::ExpectedSize::Variable
    }

    fn encoded_len(&self) -> usize {
        mem::size_of::<u8>() + mem::size_of::<u8>() + self.code.len()
    }

    fn encode_into<W: NoStdWrite>(&self, writer: &mut W) -> Result<()> {
        self.subtype.encode_into(writer)?;
        let flags = (self.code.len() as u8 - 1) << 6;
        flags.encode_into(writer)?;
        writer.write_all(&self.code)?;
        Ok(())
    }
}

impl<'dec: 'a, 'a> Decode<'dec> for NowInputEventUnicode {
    fn decode_from(cursor: &mut Cursor<'dec>) -> Result<Self> {
        let _subtype = cursor.read_u8()?;
        let flags = cursor.read_u8()?;

        let start_inclusive = cursor.position() as usize;

        let code_size = (flags >> 6) + 1;
        let end_exclusive = start_inclusive + code_size as usize;

        const SUBTYPE_FLAGS_BYTES: usize = 2;
        let bytes_left = (cursor.get_ref().len() - start_inclusive + SUBTYPE_FLAGS_BYTES) as usize;

        let code = if bytes_left == end_exclusive {
            cursor.get_ref()[start_inclusive..end_exclusive].to_vec()
        } else {
            return Err(ProtoError::new(ProtoErrorKind::Decoding(
                "NowInputEventUnicode: bytes_left != end_exclusive",
            )));
        };

        Ok(NowInputEventUnicode {
            subtype: InputMessageType::Unicode,
            code,
        })
    }
}

impl NowInputEventUnicode {
    pub fn new(code: Vec<u8>) -> Self {
        Self {
            subtype: InputMessageType::Unicode,
            code,
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum ToggleEventKeys {
    #[value = 0x0001]
    ScrollLock,
    #[value = 0x0002]
    NumLock,
    #[value = 0x0004]
    CapsLock,
    #[value = 0x0008]
    KanaLock,
    #[fallback]
    Other(u16),
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct NowInputEventToggle {
    subtype: InputMessageType,
    flags: u8,
    pub code: u16,
}

impl NowInputEventToggle {
    pub fn new_with_code(code: u16) -> Self {
        Self {
            subtype: InputMessageType::Toggle,
            flags: 0x0,
            code,
        }
    }
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct NowInputEventAction {
    subtype: InputMessageType,
    flags: u8,
    pub code: InputActionCode,
}

impl NowInputEventAction {
    pub fn new_with_code(code: InputActionCode) -> Self {
        Self {
            subtype: InputMessageType::Action,
            flags: 0x0,
            code,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "InputMessageType"]
pub enum InputEvent<'a> {
    Mouse(NowInputEventMouse),
    Scroll(NowInputEventScroll),
    Keyboard(NowInputEventKeyboard),
    Unicode(NowInputEventUnicode),
    Toggle(NowInputEventToggle),
    Action(NowInputEventAction),
    #[fallback]
    Custom(&'a [u8]),
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct NowInputMsg<'a> {
    input_event: Vec16<InputEvent<'a>>,
}

impl<'a> NowInputMsg<'a> {
    pub fn new_with_events(input_event: Vec<InputEvent<'a>>) -> Self {
        Self {
            input_event: Vec16(input_event),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::VirtChannelsCtx;
    use crate::packet::NowPacket;

    const TOGGLE_EVENT_FULL_PACKET: [u8; 10] = [0x06, 0x00, 0x43, 0x80, 0x01, 0x00, 0x05, 0x00, 0x02, 0x00];

    const MOUSE_SCROLL_EVENT_FULL_PACKET: [u8; 12] =
        [0x08, 0x00, 0x43, 0x80, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x78, 0x00];

    const KEYBOARD_EVENT_FULL_PACKET: [u8; 10] = [0x06, 0x00, 0x43, 0x80, 0x01, 0x00, 0x03, 0x01, 0x08, 0x00];

    const MOUSE_POSITION_EVENT_FULL_PACKET: [u8; 18] = [
        0x0e, 0x00, 0x43, 0x80, 0x02, 0x00, 0x01, 0x00, 0xe4, 0x05, 0x77, 0x02, 0x01, 0x00, 0xe0, 0x05, 0x70, 0x02,
    ];

    const UNICODE_EVENT_FULL_PACKET: [u8; 12] =
        [0x08, 0x00, 0x43, 0x80, 0x01, 0x00, 0x04, 0xC0, 0xe4, 0x05, 0x77, 0x02];

    #[test]
    fn input_event_mouse_encode() {
        let mouse_events = vec![
            InputEvent::Mouse(NowInputEventMouse::new_with_flags_and_position(
                EventMouseFlags::None,
                1508,
                631,
            )),
            InputEvent::Mouse(NowInputEventMouse::new_with_flags_and_position(
                EventMouseFlags::None,
                1504,
                624,
            )),
        ];

        let packet = NowPacket::from_message(NowInputMsg::new_with_events(mouse_events));

        assert_eq!(packet.encode().unwrap(), MOUSE_POSITION_EVENT_FULL_PACKET.to_vec());
    }

    #[test]
    fn input_event_mouse_decode_full_packet() {
        let mut buffer = Vec::new();
        let mut reader = std::io::Cursor::new(&MOUSE_POSITION_EVENT_FULL_PACKET[..]);
        match NowPacket::read_from(&mut reader, &mut buffer, &VirtChannelsCtx::new()) {
            Ok(_) => {}
            Err(e) => {
                e.print_trace();
                panic!("couldn't decode input mouse event packet");
            }
        }
    }

    #[test]
    fn input_event_toggle_encode() {
        let toggle_event = vec![InputEvent::Toggle(NowInputEventToggle::new_with_code(u16::from(
            ToggleEventKeys::NumLock,
        )))];

        let packet = NowPacket::from_message(NowInputMsg::new_with_events(toggle_event));

        assert_eq!(packet.encode().unwrap(), TOGGLE_EVENT_FULL_PACKET.to_vec());
    }

    #[test]
    fn input_event_toggle_decode() {
        let toggle_event = InputEvent::decode(&TOGGLE_EVENT_FULL_PACKET[6..]).unwrap();
        if let InputEvent::Toggle(toggle_event) = toggle_event {
            assert_eq!(toggle_event.subtype, InputMessageType::Toggle);
            assert_eq!(toggle_event.code, 2);
        } else {
            panic!("couldn't decode toggle message")
        }
    }

    #[test]
    fn input_event_keyboard_encode() {
        let kb_events = vec![InputEvent::Keyboard(NowInputEventKeyboard::new_with_flags_and_code(
            1, 8,
        ))];

        let packet = NowPacket::from_message(NowInputMsg::new_with_events(kb_events));

        assert_eq!(packet.encode().unwrap(), KEYBOARD_EVENT_FULL_PACKET.to_vec());
    }

    #[test]
    fn input_event_keyboard_decode() {
        let kb_event = InputEvent::decode(&KEYBOARD_EVENT_FULL_PACKET[6..]).unwrap();
        if let InputEvent::Keyboard(kb_event) = kb_event {
            assert_eq!(kb_event.subtype, InputMessageType::Keyboard);
            assert_eq!(kb_event.code, 8);
        } else {
            panic!("couldn't decode keyboard message")
        }
    }

    #[test]
    fn input_event_scroll_encode() {
        let scroll_events = vec![InputEvent::Scroll(NowInputEventScroll::new_with_position(0, 120))];

        let packet = NowPacket::from_message(NowInputMsg::new_with_events(scroll_events));

        assert_eq!(packet.encode().unwrap(), MOUSE_SCROLL_EVENT_FULL_PACKET.to_vec());
    }

    #[test]
    fn input_event_scroll_decode() {
        let scroll_event = InputEvent::decode(&MOUSE_SCROLL_EVENT_FULL_PACKET[6..]).unwrap();
        if let InputEvent::Scroll(scroll_event) = scroll_event {
            assert_eq!(scroll_event.subtype, InputMessageType::Scroll);
            assert_eq!(scroll_event.x, 0);
            assert_eq!(scroll_event.y, 120);
        } else {
            panic!("couldn't decode scroll message")
        }
    }

    #[test]
    fn input_event_unicode_encode() {
        let unicode_events = vec![InputEvent::Unicode(NowInputEventUnicode::new(vec![
            0xe4, 0x05, 0x77, 0x02,
        ]))];

        let packet = NowPacket::from_message(NowInputMsg::new_with_events(unicode_events));
        assert_eq!(packet.encode().unwrap(), UNICODE_EVENT_FULL_PACKET.to_vec());
    }

    #[test]
    fn input_event_unicode_decode() {
        let unicode_event = InputEvent::decode(&UNICODE_EVENT_FULL_PACKET[6..]).unwrap();
        if let InputEvent::Unicode(unicode_event) = unicode_event {
            assert_eq!(unicode_event.subtype, InputMessageType::Unicode);
            assert_eq!(unicode_event.code, vec![0xe4, 0x05, 0x77, 0x02]);
        } else {
            panic!("didnt decode unicode message")
        }
    }
}
