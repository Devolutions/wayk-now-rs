#![no_main]

use std::io::Cursor;
use libfuzzer_sys::fuzz_target;
use wayk_proto::message::{NowMessage, MessageType};

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);
    let _ = NowMessage::decode_from(MessageType::Status, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Handshake, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Negotiate, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Authenticate, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Associate, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Capabilities, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Channel, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Activate, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Terminate, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Surface, &mut cursor);
    cursor.set_position(0);
    
    let _ = NowMessage::decode_from(MessageType::Update, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Input, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Mouse, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Network, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Access, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Desktop, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::System, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Session, &mut cursor);
    cursor.set_position(0);

    let _ = NowMessage::decode_from(MessageType::Sharing, &mut cursor);
    cursor.set_position(0);
});
