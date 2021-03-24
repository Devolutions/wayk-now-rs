// NOW_HANDSHAKE_MSG

use crate::message::status::{HandshakeStatusCode, NowStatus};
use crate::version::{WAYK_NOW_VERSION_MAJOR, WAYK_NOW_VERSION_MINOR, WAYK_NOW_VERSION_PATCH};

__flags_struct! {
    HanshakeFlags: u32 => {
        failure = FAILURE = 0x0000_0001,
        reconnect = RECONNECT = 0x0000_0002,
        reserved1 = RESERVED1 = 0x0000_0004,
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowHandshakeMsg {
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    reserved1: u8,
    pub flags: HanshakeFlags,
    pub status: NowStatus<HandshakeStatusCode>,
    reserved2: u16,
    reserved3: u16,
    pub cookie: [u32; 4],
    pub session_id: u32,
    session_flags: u32,
}

impl NowHandshakeMsg {
    pub const REQUIRED_SIZE: usize = 40;

    pub fn new_success() -> Self {
        Self::default()
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn configure_failure(&mut self, status: NowStatus<HandshakeStatusCode>) {
        self.flags.set_failure();
        self.status = status;
    }

    pub fn configure_reconnect(&mut self, cookie: [u32; 4], session_id: u32) {
        self.flags.set_reconnect();
        self.cookie = cookie;
        self.session_id = session_id;
    }
}

impl Default for NowHandshakeMsg {
    fn default() -> Self {
        NowHandshakeMsg {
            version_major: WAYK_NOW_VERSION_MAJOR,
            version_minor: WAYK_NOW_VERSION_MINOR,
            version_patch: WAYK_NOW_VERSION_PATCH,
            reserved1: 0,
            flags: HanshakeFlags::new_empty(),
            status: NowStatus::default(),
            reserved2: 0,
            reserved3: 0,
            cookie: [0, 0, 0, 0],
            session_id: 0,
            session_flags: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    #[rustfmt::skip]
    const HANDSHAKE_MSG_SUCCESS: [u8; 40] = [
        WAYK_NOW_VERSION_MAJOR, // major
        WAYK_NOW_VERSION_MINOR, // minor
        WAYK_NOW_VERSION_PATCH, // patch
        // reserved1
        0x00,
        // flags
        0x00, 0x00, 0x00, 0x00,
        // status
        0x00, 0x00, 0x00, 0x00,
        // reserved2
        0x00, 0x00,
        // reserved3
        0x00, 0x00,
        // cookie
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // sessionId
        0x00, 0x00, 0x00, 0x00,
        // sessionFlags
        0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn decoding() {
        let msg = NowHandshakeMsg::decode(&HANDSHAKE_MSG_SUCCESS).unwrap();
        assert_eq!(msg.version_major, WAYK_NOW_VERSION_MAJOR);
        assert_eq!(msg.version_minor, WAYK_NOW_VERSION_MINOR);
        assert_eq!(msg.version_patch, WAYK_NOW_VERSION_PATCH);
        assert!(!msg.flags.failure());
    }

    #[test]
    fn encoding() {
        let msg = NowHandshakeMsg::new_success();
        assert_eq!(HANDSHAKE_MSG_SUCCESS.to_vec(), msg.encode().unwrap());
    }
}
