// NOW_NEGOTIATE_MSG

use crate::container::Vec8;
use crate::message::AuthType;

__flags_struct! {
    NegotiateFlags: u32 => {
        srp_extended = SRP_EXTENDED = 0x0000_0001,
    }
}

#[derive(Encode, Decode, Debug, Clone)]
pub struct NowNegotiateMsg {
    pub flags: NegotiateFlags,
    pub auth_list: Vec8<AuthType>,
}

impl Default for NowNegotiateMsg {
    fn default() -> Self {
        Self::new(NegotiateFlags::new_empty().set_srp_extended())
    }
}

impl NowNegotiateMsg {
    pub const REQUIRED_SIZE: usize = 5;

    pub fn new(flags: NegotiateFlags) -> Self {
        NowNegotiateMsg::new_with_auth_list(flags, Vec::new())
    }

    pub fn new_with_auth_list(flags: NegotiateFlags, auth_list: Vec<AuthType>) -> Self {
        NowNegotiateMsg {
            flags,
            auth_list: Vec8(auth_list),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    #[rustfmt::skip]
    const NEGOTIATE_MSG: [u8; 9] = [
        0x01, 0x00, 0x00, 0x00, // flags
        0x04, // auth count
        // auth list
        0x01, 0x02, 0x08, 0x04,
    ];

    #[test]
    fn decoding() {
        let msg = NowNegotiateMsg::decode(&NEGOTIATE_MSG).unwrap();
        assert!(msg.flags.srp_extended());
        assert_eq!(msg.auth_list.len(), 4);
        assert_eq!(msg.auth_list[0], AuthType::PFP);
        assert_eq!(msg.auth_list[1], AuthType::SRP);
        assert_eq!(msg.auth_list[2], AuthType::SRD);
        assert_eq!(msg.auth_list[3], AuthType::NTLM);
    }

    #[test]
    fn encoding() {
        let mut msg = NowNegotiateMsg::default();
        msg.auth_list.push(AuthType::PFP);
        msg.auth_list.push(AuthType::SRP);
        msg.auth_list.push(AuthType::SRD);
        msg.auth_list.push(AuthType::NTLM);
        assert_eq!(msg.encode().unwrap(), NEGOTIATE_MSG.to_vec());
    }
}
