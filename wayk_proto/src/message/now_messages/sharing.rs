use crate::message::NowString256;
use num_derive::FromPrimitive;
use std::str::FromStr;

#[derive(Encode, Decode, FromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum SharingMessageType {
    Suspend = 0x01,
    Resume = 0x02,
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowSharingSuspendMsg {
    subtype: SharingMessageType,
    flags: u8,
    reserved: u16,
    pub message: NowString256,
}

impl Default for NowSharingSuspendMsg {
    fn default() -> Self {
        NowSharingSuspendMsg {
            subtype: SharingMessageType::Suspend,
            flags: 0,
            reserved: 0,
            message: NowString256::from_str("").unwrap(),
        }
    }
}

impl NowSharingSuspendMsg {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_message(message: NowString256) -> Self {
        Self {
            message,
            ..Self::default()
        }
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowSharingResumeMsg {
    subtype: SharingMessageType,
    flags: u8,
    reserved: u16,
}

// NOW_SHARING_MSG

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "SharingMessageType"]
pub enum NowSharingMsg {
    Suspend(NowSharingSuspendMsg),
    Resume(NowSharingResumeMsg),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};
    use std::str::FromStr;

    #[rustfmt::skip]
    const NOW_SHARING_SUSPEND_MSG: [u8; 6] = [
        0x01, // subtype
        0x00, // flags
        0x00, 0x00, // reserved
        0x00, 0x00, // message
    ];

    #[test]
    fn suspend_decoding() {
        let msg = NowSharingMsg::decode(&NOW_SHARING_SUSPEND_MSG).unwrap();
        if let NowSharingMsg::Suspend(msg) = msg {
            assert_eq!(msg.subtype, SharingMessageType::Suspend);
            assert_eq!(msg.flags, 0);
            assert_eq!(msg.reserved, 0);
            assert_eq!(msg.message, "");
        } else {
            panic!("expected a surface list req message and got {:?}", msg);
        }
    }

    #[test]
    fn suspend_encoding() {
        let msg = NowSharingSuspendMsg::new_with_message(NowString256::from_str("").unwrap());
        assert_eq!(msg.encode().unwrap(), NOW_SHARING_SUSPEND_MSG.to_vec());
    }
}
