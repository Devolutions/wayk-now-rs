// NOW_ASSOCIATE_MSG

use crate::message::status::{AssociateStatusCode, NowStatus};

#[derive(Decode, Encode, Debug, PartialEq, Clone, Copy)]
pub enum AssociateMessageType {
    #[value = 0x01]
    Info,
    #[value = 0x02]
    Request,
    #[value = 0x03]
    Response,
    #[fallback]
    Other(u8),
}

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "AssociateMessageType"]
pub enum NowAssociateMsg<'a> {
    Info(NowAssociateInfoMsg),
    Request(NowAssociateRequestMsg),
    Response(NowAssociateResponseMsg),
    #[fallback]
    Custom(&'a [u8]),
}

impl NowAssociateMsg<'_> {
    pub fn new_info() -> Self {
        Self::Info(NowAssociateInfoMsg::default())
    }

    pub fn new_request() -> Self {
        Self::Request(NowAssociateRequestMsg::default())
    }

    pub fn new_response() -> Self {
        Self::Response(NowAssociateResponseMsg::default())
    }

    pub fn new_response_with_status(status: NowStatus<AssociateStatusCode>) -> Self {
        Self::Response(NowAssociateResponseMsg {
            status,
            ..NowAssociateResponseMsg::default()
        })
    }
}

impl From<NowAssociateInfoMsg> for NowAssociateMsg<'_> {
    fn from(msg: NowAssociateInfoMsg) -> Self {
        Self::Info(msg)
    }
}

impl From<NowAssociateRequestMsg> for NowAssociateMsg<'_> {
    fn from(msg: NowAssociateRequestMsg) -> Self {
        Self::Request(msg)
    }
}

impl From<NowAssociateResponseMsg> for NowAssociateMsg<'_> {
    fn from(msg: NowAssociateResponseMsg) -> Self {
        Self::Response(msg)
    }
}

// subtypes

__flags_struct! {
    AssociateInfoFlags: u16 => {
        active = ACTIVE = 0x0001,
        failure = FAILURE = 0x8000,
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowAssociateInfoMsg {
    subtype: AssociateMessageType,
    reserved: u8,
    pub flags: AssociateInfoFlags,
    pub session_id: u32,
}

impl Default for NowAssociateInfoMsg {
    fn default() -> Self {
        Self::new(AssociateInfoFlags::new_empty())
    }
}

impl NowAssociateInfoMsg {
    pub const SUBTYPE: AssociateMessageType = AssociateMessageType::Info;

    pub fn new(flags: AssociateInfoFlags) -> Self {
        Self::new_with_session_id(flags, 0x0000_0000)
    }

    pub fn new_with_session_id(flags: AssociateInfoFlags, session_id: u32) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            reserved: 0,
            flags,
            session_id,
        }
    }
}

__flags_struct! {
    AssociateRequestFlags: u16 => {
        force = FORCE = 0x0001,
        failure = FAILURE = 0x8000,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowAssociateRequestMsg {
    subtype: AssociateMessageType,
    reserved: u8,
    pub flags: AssociateRequestFlags,
    pub session_id: u32,
}

impl Default for NowAssociateRequestMsg {
    fn default() -> Self {
        Self::new(AssociateRequestFlags::new_empty())
    }
}

impl NowAssociateRequestMsg {
    pub const SUBTYPE: AssociateMessageType = AssociateMessageType::Request;

    pub fn new(flags: AssociateRequestFlags) -> Self {
        Self::new_with_session_id(flags, 0x0000_0000)
    }

    pub fn new_with_session_id(flags: AssociateRequestFlags, session_id: u32) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            reserved: 0x00,
            flags,
            session_id,
        }
    }
}

__flags_struct! {
    AssociateResponseFlags: u16 => {
        failure = FAILURE = 0x8000,
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowAssociateResponseMsg {
    subtype: AssociateMessageType,
    reserved: u8,
    pub flags: AssociateResponseFlags,
    pub session_id: u32,
    pub status: NowStatus<AssociateStatusCode>,
}

impl Default for NowAssociateResponseMsg {
    fn default() -> Self {
        Self::new(AssociateResponseFlags::new_empty(), NowStatus::default())
    }
}

impl NowAssociateResponseMsg {
    pub const SUBTYPE: AssociateMessageType = AssociateMessageType::Response;

    pub fn new(flags: AssociateResponseFlags, status: NowStatus<AssociateStatusCode>) -> Self {
        Self::new_with_session_id(flags, status, 0x0000_0000)
    }

    pub fn new_with_session_id(
        flags: AssociateResponseFlags,
        status: NowStatus<AssociateStatusCode>,
        session_id: u32,
    ) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            reserved: 0x00,
            flags,
            session_id,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    #[test]
    fn decoding_with_subtype_check() {
        let msg = NowAssociateMsg::decode(&ASSOCIATE_MSG_REQUEST).unwrap();
        if let NowAssociateMsg::Request(msg) = msg {
            assert_eq!(msg.subtype, AssociateMessageType::Request);
            assert_eq!(msg.reserved, 0x00);
            assert_eq!(msg.flags, 0x0000);
            assert_eq!(msg.session_id, 0x0000_0000);
        } else {
            panic!("Expected a request message, found {:?}", msg);
        }
    }

    const ASSOCIATE_MSG_INFO: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[test]
    fn info_decoding() {
        let msg = NowAssociateInfoMsg::decode(&ASSOCIATE_MSG_INFO).unwrap();
        assert_eq!(msg.subtype, AssociateMessageType::Info);
        assert_eq!(msg.reserved, 0x00);
        assert_eq!(msg.flags, 0x0000);
        assert_eq!(msg.session_id, 0x0000_0000);
    }

    #[test]
    fn associate_info_encoding() {
        let request = NowAssociateMsg::new_info();
        assert_eq!(ASSOCIATE_MSG_INFO, request.encode().unwrap()[0..]);
    }

    const ASSOCIATE_MSG_REQUEST: [u8; 8] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[test]
    fn request_decoding() {
        let msg = NowAssociateInfoMsg::decode(&ASSOCIATE_MSG_REQUEST).unwrap();
        assert_eq!(msg.subtype, AssociateMessageType::Request);
        assert_eq!(msg.reserved, 0x00);
        assert_eq!(msg.flags, 0x0000);
        assert_eq!(msg.session_id, 0x0000_0000);
    }

    #[test]
    fn request_encoding() {
        let request = NowAssociateMsg::new_request();
        assert_eq!(ASSOCIATE_MSG_REQUEST, request.encode().unwrap()[0..]);
    }

    const ASSOCIATE_MSG_RESPONSE: [u8; 12] = [0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[test]
    fn response_decoding() {
        let msg = NowAssociateResponseMsg::decode(&ASSOCIATE_MSG_RESPONSE).unwrap();
        assert_eq!(msg.subtype, AssociateMessageType::Response);
        assert_eq!(msg.reserved, 0x00);
        assert_eq!(msg.flags, 0x0000);
        assert_eq!(msg.session_id, 0x0000_0000);
        assert_eq!(msg.status, 0x0000_0000);
    }

    #[test]
    fn response_encoding() {
        let request = NowAssociateMsg::new_response();
        assert_eq!(ASSOCIATE_MSG_RESPONSE, request.encode().unwrap()[0..]);
    }
}
