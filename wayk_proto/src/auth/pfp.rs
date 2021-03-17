use crate::error::Result;
use crate::message::{AuthType, NowAuthenticateMsg, NowAuthenticateTokenMsgOwned, NowString256, NowString64};
use crate::serialization::Encode;
use core::str::FromStr;

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum PFPMessageType {
    #[value = 0x01]
    Negotiate,
    #[value = 0x02]
    Challenge,
    #[value = 0x03]
    Response,
    #[fallback]
    Other(u16),
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum PFPMessageFlags {
    #[value = 0x0000]
    NoChallenge,
    #[value = 0x0001]
    Question,
    #[fallback]
    Other(u16),
}

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "PFPMessageType"]
pub enum NowAuthPFP {
    Negotiate(NowAuthPFPNegotiate),
    Challenge(NowAuthPFPChallenge),
    Response(NowAuthPFPResponse),
}

impl NowAuthPFP {
    pub fn new_owned_negotiate_token<'a>(friendly_name: &str, friendly_text: &str) -> Result<NowAuthenticateMsg<'a>> {
        let negotiate_token = NowAuthPFPNegotiate::new(
            NowString64::from_str(friendly_name)?,
            NowString256::from_str(friendly_text)?,
        )
        .encode()?;

        Ok(NowAuthenticateTokenMsgOwned::new(AuthType::PFP, negotiate_token).into())
    }
}

// pfp types
#[derive(Decode, Encode, Debug, Clone)]
pub struct NowAuthPFPNegotiate {
    pub subtype: PFPMessageType,
    pub flags: PFPMessageFlags,
    pub friendly_name: NowString64,
    pub friendly_text: NowString256,
}

impl NowAuthPFPNegotiate {
    pub const MIN_REQUIRED_SIZE: usize = 6;

    pub fn new(friendly_name: NowString64, friendly_text: NowString256) -> Self {
        Self {
            subtype: PFPMessageType::Negotiate,
            flags: PFPMessageFlags::Question,
            friendly_name,
            friendly_text,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowAuthPFPChallenge {
    pub subtype: PFPMessageType,
    pub flags: PFPMessageFlags,
    pub question: NowString256,
}

impl NowAuthPFPChallenge {
    pub const MIN_REQUIRED_SIZE: usize = 5;

    pub fn new_without_question() -> Self {
        Self {
            subtype: PFPMessageType::Challenge,
            flags: PFPMessageFlags::NoChallenge,
            question: NowString256::new_empty(),
        }
    }

    pub fn new_with_question(question: NowString256) -> Self {
        Self {
            subtype: PFPMessageType::Challenge,
            flags: PFPMessageFlags::Question,
            question,
        }
    }
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct NowAuthPFPResponse {
    pub subtype: PFPMessageType,
    pub flags: PFPMessageFlags,
    pub answer: NowString256,
}

impl NowAuthPFPResponse {
    pub const MIN_REQUIRED_SIZE: usize = 5;

    pub fn new(answer: NowString256) -> Self {
        Self {
            subtype: PFPMessageType::Response,
            flags: PFPMessageFlags::Question,
            answer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};
    use core::str::FromStr;

    #[rustfmt::skip]
    const PFP_NEGOTIATE_TOKEN: [u8; 26] = [
        0x01, 0x00, // type
        0x01, 0x00, // flag
        0x0a, // friendly name len
        0x4a, 0x6f, 0x68, 0x6e, 0x6e, 0x79, 0x20, 0x44, 0x6f, 0x65, 0x00, // nul terminator
        0x08, // friendly text len
        0x49, 0x74, 0x27, 0x73, 0x20, 0x6d, 0x65, 0x2e,
        0x00, // null terminator
    ];

    #[test]
    fn negotiate_decoding() {
        let msg = NowAuthPFP::decode(&PFP_NEGOTIATE_TOKEN).unwrap();
        if let NowAuthPFP::Negotiate(msg) = msg {
            assert_eq!(msg.subtype, PFPMessageType::Negotiate);
            assert_eq!(msg.flags, PFPMessageFlags::Question);
            assert_eq!(msg.friendly_name, "Johnny Doe");
            assert_eq!(msg.friendly_text, "It's me.");
        } else {
            panic!("Expected a negotiate message, found {:?}", msg);
        }
    }

    #[test]
    fn negotiate_encoding() {
        let msg = NowAuthPFPNegotiate::new(
            NowString64::from_str("Johnny Doe").unwrap(),
            NowString256::from_str("It's me.").unwrap(),
        );
        assert_eq!(msg.encode().unwrap(), PFP_NEGOTIATE_TOKEN.to_vec());
    }

    #[rustfmt::skip]
    const PFP_CHALLENGE_TOKEN: [u8; 18] = [
        0x02, 0x00, // type
        0x01, 0x00, // flag
        0x0c, // question len
        0x48, 0x6f, 0x77, 0x20, 0x61, 0x72, 0x65, 0x20, 0x79, 0x6f, 0x75, 0x3f,
        0x00, // null terminator
    ];

    #[rustfmt::skip]
    const PFP_NO_CHALLENGE_TOKEN: [u8; 6] = [
        0x02, 0x00, // type
        0x00, 0x00, // flag
        0x00, 0x00,
    ];

    #[test]
    fn challenge_decoding() {
        let msg = NowAuthPFP::decode(&PFP_CHALLENGE_TOKEN).unwrap();
        if let NowAuthPFP::Challenge(msg) = msg {
            assert_eq!(msg.subtype, PFPMessageType::Challenge);
            assert_eq!(msg.flags, PFPMessageFlags::Question);
            assert_eq!(msg.question, "How are you?");
        } else {
            panic!("Expected a challenge message, found {:?}", msg);
        }
    }

    #[test]
    fn no_challenge_decoding() {
        let msg = NowAuthPFP::decode(&PFP_NO_CHALLENGE_TOKEN).unwrap();
        if let NowAuthPFP::Challenge(msg) = msg {
            assert_eq!(msg.subtype, PFPMessageType::Challenge);
            assert_eq!(msg.flags, PFPMessageFlags::NoChallenge);
            assert_eq!(msg.question, "");
        } else {
            panic!("Expected a challenge message, found {:?}", msg);
        }
    }

    #[test]
    fn challenge_encoding() {
        let msg = NowAuthPFPChallenge::new_with_question(NowString256::from_str("How are you?").unwrap());
        assert_eq!(msg.encode().unwrap(), PFP_CHALLENGE_TOKEN.to_vec());
    }

    #[test]
    fn no_challenge_encoding() {
        let msg = NowAuthPFPChallenge::new_without_question();
        assert_eq!(PFP_NO_CHALLENGE_TOKEN.to_vec(), msg.encode().unwrap());
    }

    #[rustfmt::skip]
    const PFP_RESPONSE_TOKEN: [u8; 12] = [
        0x03, 0x00, // type
        0x01, 0x00, // flag
        // answer
        0x6, 0xe5, 0x85, 0x83, 0xe6, 0xb0, 0x97, 0x00,
    ];

    #[test]
    fn response_decoding() {
        let msg = NowAuthPFP::decode(&PFP_RESPONSE_TOKEN).unwrap();
        if let NowAuthPFP::Response(msg) = msg {
            assert_eq!(msg.subtype, PFPMessageType::Response);
            assert_eq!(msg.flags, PFPMessageFlags::Question);
            assert_eq!(msg.answer, "元気");
        } else {
            panic!("Expected a response message, found {:?}", msg);
        }
    }

    #[test]
    fn response_encoding() {
        let msg = NowAuthPFPResponse::new(NowString256::from_str("元気").unwrap());
        assert_eq!(msg.encode().unwrap(), PFP_RESPONSE_TOKEN.to_vec());
    }
}
