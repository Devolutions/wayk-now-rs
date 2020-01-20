use crate::{
    container::{Bytes16, Vec16},
    message::status::{AuthStatusCode, NowStatus},
};
use num_derive::FromPrimitive;

// TODO: check usage of this enum...
// SRP message types
#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum SRPMessageType {
    SRPInitiate = 0x01,
    SRPOffer = 0x02,
    SRPAccept = 0x03,
    SRPConfirm = 0x04,
}

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum AuthenticateMessageType {
    Token = 0x01,
    Success = 0x02,
    Failure = 0x03,
}

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum AuthType {
    None = 0x00,
    PFP = 0x01,
    SRP = 0x02,
    IGNORED1 = 0x03,
    NTLM = 0x04,
    SPNEGO = 0x05,
    Kerberos = 0x06,
    CredSSP = 0x07,
    SRD = 0x08,
}

__flags_struct! {
    AuthentificationFailureFlags: u8 => {
        retry = RETRY = 0x01,
    }
}

// NOW_AUTHENTICATE_MSG

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "AuthenticateMessageType"]
pub enum NowAuthenticateMsg<'a> {
    Token(NowAuthenticateTokenMsg<'a>),
    Success(NowAuthenticateSuccessMsg),
    Failure(NowAuthenticateFailureMsg),

    #[decode_ignore]
    OwnedToken(NowAuthenticateTokenMsgOwned),
}

impl<'a> From<NowAuthenticateTokenMsg<'a>> for NowAuthenticateMsg<'a> {
    fn from(msg: NowAuthenticateTokenMsg<'a>) -> Self {
        Self::Token(msg)
    }
}

impl From<NowAuthenticateTokenMsgOwned> for NowAuthenticateMsg<'_> {
    fn from(msg: NowAuthenticateTokenMsgOwned) -> Self {
        Self::OwnedToken(msg)
    }
}

impl From<NowAuthenticateSuccessMsg> for NowAuthenticateMsg<'_> {
    fn from(msg: NowAuthenticateSuccessMsg) -> Self {
        Self::Success(msg)
    }
}

impl From<NowAuthenticateFailureMsg> for NowAuthenticateMsg<'_> {
    fn from(msg: NowAuthenticateFailureMsg) -> Self {
        Self::Failure(msg)
    }
}

// subtypes

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowAuthenticateTokenMsg<'a> {
    subtype: AuthenticateMessageType,
    flags: u8,
    pub auth_type: AuthType,
    auth_flags: u8,
    pub token_data: Bytes16<'a>,
}

impl<'a> NowAuthenticateTokenMsg<'a> {
    pub const SUBTYPE: AuthenticateMessageType = AuthenticateMessageType::Token;
    pub const REQUIRED_SIZE: usize = 6;

    pub fn new(auth_type: AuthType, token_data: &'a [u8]) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            auth_type,
            auth_flags: 0,
            token_data: Bytes16(token_data),
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowAuthenticateTokenMsgOwned {
    subtype: AuthenticateMessageType,
    flags: u8,
    pub auth_type: AuthType,
    auth_flags: u8,
    pub token_data: Vec16<u8>,
}

impl NowAuthenticateTokenMsgOwned {
    pub const SUBTYPE: AuthenticateMessageType = AuthenticateMessageType::Token;

    pub fn new(auth_type: AuthType, token_data: Vec<u8>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            auth_type,
            auth_flags: 0,
            token_data: Vec16(token_data),
        }
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowAuthenticateSuccessMsg {
    subtype: AuthenticateMessageType,
    flags: u8,
    reserved: u16,
    pub session_id: u32,
    pub cookie: [u32; 4],
}

impl NowAuthenticateSuccessMsg {
    pub const SUBTYPE: AuthenticateMessageType = AuthenticateMessageType::Success;
    pub const REQUIRED_SIZE: usize = 24;

    pub fn new(session_id: u32, cookie: [u32; 4]) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: 0,
            reserved: 0,
            session_id,
            cookie,
        }
    }
}

impl Default for NowAuthenticateSuccessMsg {
    fn default() -> Self {
        Self::new(0, [0, 0, 0, 0])
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowAuthenticateFailureMsg {
    subtype: AuthenticateMessageType,
    pub flags: AuthentificationFailureFlags,
    reserved: u16,
    pub status: NowStatus<AuthStatusCode>,
}

impl NowAuthenticateFailureMsg {
    pub const SUBTYPE: AuthenticateMessageType = AuthenticateMessageType::Failure;
    pub const REQUIRED_SIZE: usize = 5;

    pub fn new(flags: AuthentificationFailureFlags, status: NowStatus<AuthStatusCode>) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags,
            reserved: 0,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        message::status::{AuthStatusCode, SeverityLevel, StatusType},
        serialization::{Decode, Encode},
    };

    #[test]
    fn decoding_with_subtype_check() {
        let msg = NowAuthenticateMsg::decode(&AUTHENTICATE_TOKEN_MSG).unwrap();
        if let NowAuthenticateMsg::Token(msg) = msg {
            assert_eq!(msg.subtype, AuthenticateMessageType::Token);
            assert_eq!(msg.flags, 0x00);
            assert_eq!(msg.auth_type, AuthType::SRP);
            assert_eq!(msg.token_data.len(), 281);
        } else {
            panic!("Expected a token message, found {:?}", msg);
        }
    }

    #[rustfmt::skip]
    const AUTHENTICATE_TOKEN_MSG: [u8; 287] = [
        0x01, // subtype
        0x00, // flags
        0x02, // auth type
        0x00, // auth flags
        0x19, 0x01, // token size
        // token data
        0x53, 0x52, 0x50, 0x00, 0x01, 0x06, 0x00, 0x00, 0x00, 0x01, 0x12, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x04, 0x00, 0x77, 0x61, 0x79, 0x6b, 0x00, 0x00, 0x01, 0x4c, 0x93, 0x3f, 0xdd, 0x99,
        0x1c, 0xe1, 0x8d, 0x05, 0xde, 0xb9, 0x93, 0x2c, 0x21, 0xf1, 0xe8, 0xbc, 0x34, 0x1f, 0xeb,
        0xbf, 0xb1, 0x3d, 0x12, 0x59, 0x8b, 0x3f, 0xa2, 0x65, 0xf0, 0x94, 0x55, 0xb8, 0x1a, 0xd4,
        0x12, 0x4e, 0x18, 0x2f, 0xcd, 0x9e, 0xa3, 0xb3, 0x5e, 0x66, 0x0a, 0x85, 0x5a, 0x35, 0x21,
        0x97, 0x53, 0x4a, 0xbe, 0x20, 0xed, 0xd9, 0x95, 0xdb, 0x57, 0x5f, 0x0e, 0x79, 0xaf, 0xa0,
        0x75, 0x24, 0x40, 0x77, 0xea, 0x81, 0xb5, 0xea, 0x6c, 0xf2, 0xbc, 0x37, 0x28, 0x86, 0x91,
        0x24, 0x74, 0xf8, 0xb5, 0x75, 0xea, 0x21, 0x6d, 0x37, 0x76, 0x6a, 0x63, 0x4c, 0x08, 0x3b,
        0x88, 0xea, 0xd8, 0x76, 0xe2, 0x1e, 0x58, 0x1a, 0xef, 0x22, 0xb6, 0x36, 0x09, 0x0a, 0x58,
        0x8a, 0x44, 0x7f, 0x52, 0x07, 0x6c, 0x5e, 0x56, 0x54, 0x64, 0x3c, 0x3c, 0x7d, 0x3d, 0x11,
        0x83, 0x7c, 0xa0, 0x78, 0x3c, 0x4a, 0xa4, 0xad, 0xbd, 0x2f, 0x06, 0x0a, 0x69, 0x1c, 0xac,
        0x8c, 0xdc, 0xe4, 0xf8, 0x31, 0x69, 0xac, 0xbf, 0x1f, 0x2a, 0x07, 0x2b, 0x67, 0xd1, 0x43,
        0xba, 0x1c, 0xcd, 0xf8, 0xa3, 0xbe, 0xd4, 0x40, 0x68, 0x60, 0x8b, 0x9c, 0x9b, 0xc5, 0x6b,
        0x0f, 0xbb, 0x6f, 0xae, 0x3e, 0xbd, 0xa1, 0x96, 0x0f, 0x2c, 0xa8, 0x5a, 0xfc, 0xea, 0xdc,
        0x75, 0x5b, 0x22, 0xea, 0xab, 0x6c, 0xd9, 0xe2, 0xa1, 0xa8, 0x55, 0xb9, 0xda, 0x7b, 0x52,
        0x7c, 0x15, 0x63, 0xc2, 0x83, 0x61, 0xcd, 0x3b, 0x18, 0xa1, 0x89, 0x3a, 0x4f, 0xce, 0xee,
        0x8d, 0x80, 0xfa, 0x79, 0x2f, 0xf0, 0x49, 0x21, 0x49, 0x55, 0xd5, 0x7a, 0x74, 0xbc, 0x36,
        0xbe, 0xd7, 0xf0, 0x06, 0x23, 0x2c, 0x0f, 0xc0, 0x4a, 0xb0, 0x3b, 0x28, 0xec, 0x0a, 0xdd,
        0xe7, 0x08, 0x13, 0x0a, 0x35, 0xd5, 0x38, 0x88, 0x3d, 0x1d, 0x55,
    ];

    #[test]
    fn token_decoding() {
        let msg = NowAuthenticateTokenMsg::decode(&AUTHENTICATE_TOKEN_MSG).unwrap();
        assert_eq!(msg.subtype, AuthenticateMessageType::Token);
        assert_eq!(msg.flags, 0x00);
        assert_eq!(msg.auth_type, AuthType::SRP);
        assert_eq!(msg.token_data.len(), 281);
    }

    #[test]
    fn token_encoding() {
        let msg = NowAuthenticateTokenMsg::new(AuthType::SRP, &AUTHENTICATE_TOKEN_MSG[6..]);
        assert_eq!(msg.encode().unwrap(), AUTHENTICATE_TOKEN_MSG.to_vec());
    }

    #[rustfmt::skip]
    const AUTHENTICATE_SUCCESS_MSG: [u8; 24] = [
        0x02, // subtype
        0x00, // flags
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // session id
        // cookie
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];

    #[test]
    fn success_decoding() {
        let msg = NowAuthenticateSuccessMsg::decode(&AUTHENTICATE_SUCCESS_MSG).unwrap();
        assert_eq!(msg.subtype, AuthenticateMessageType::Success);
        assert_eq!(msg.flags, 0x00);
        assert_eq!(msg.reserved, 0x0000);
        assert_eq!(msg.session_id, 0x0000_0000);
        assert_eq!(msg.cookie, [0, 0, 0, 0]);
    }

    #[test]
    fn success_encoding() {
        let msg = NowAuthenticateSuccessMsg::default();
        assert_eq!(msg.encode().unwrap(), AUTHENTICATE_SUCCESS_MSG.to_vec());
    }

    #[rustfmt::skip]
    const AUTHENTICATE_FAILURE_MSG: [u8; 8] = [
        0x03, // subtype
        0x01, // flags
        0x00, 0x00, // reserved
        0xff, 0xff, 0x17, 0x80, // status
    ];

    #[test]
    fn failure_decoding() {
        let msg = NowAuthenticateFailureMsg::decode(&AUTHENTICATE_FAILURE_MSG).unwrap();
        assert_eq!(msg.subtype, AuthenticateMessageType::Failure);
        assert_eq!(msg.flags.value, 0x01);
        assert!(msg.flags.retry());
        assert_eq!(msg.reserved, 0x0000);
        assert_eq!(msg.status.severity(), SeverityLevel::Error);
        assert_eq!(msg.status.code(), AuthStatusCode::Failure);
        assert_eq!(msg.status.status_type(), StatusType::Auth);
    }

    #[test]
    fn failure_encoding() {
        let nstatus = NowStatus::builder(AuthStatusCode::Failure)
            .severity(SeverityLevel::Error)
            .status_type(StatusType::Auth)
            .build();
        let msg = NowAuthenticateFailureMsg::new(AuthentificationFailureFlags::new_empty().set_retry(), nstatus);
        assert_eq!(msg.encode().unwrap(), AUTHENTICATE_FAILURE_MSG.to_vec());
    }
}
