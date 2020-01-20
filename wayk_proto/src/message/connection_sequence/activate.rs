#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub struct NowActivateMsg {
    flags: u32,
}

impl Default for NowActivateMsg {
    fn default() -> Self {
        NowActivateMsg { flags: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    const ACTIVATE_MSG: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

    #[test]
    fn decoding() {
        let msg = NowActivateMsg::decode(&ACTIVATE_MSG).unwrap();
        assert_eq!(msg.flags, 0);
    }

    #[test]
    fn encoding() {
        let msg = NowActivateMsg::default();
        assert_eq!(msg.encode().unwrap(), ACTIVATE_MSG.to_vec());
    }
}
