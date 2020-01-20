use crate::message::status::{DisconnectStatusCode, NowStatus};

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct NowTerminateMsg {
    flags: u32,
    pub status: NowStatus<DisconnectStatusCode>,
}

impl Default for NowTerminateMsg {
    fn default() -> Self {
        Self::new(NowStatus::default())
    }
}

impl NowTerminateMsg {
    pub fn new(status: NowStatus<DisconnectStatusCode>) -> Self {
        Self { flags: 0, status }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        message::status::{SeverityLevel, StatusType},
        serialization::{Decode, Encode},
    };

    #[rustfmt::skip]
    const TERMINATE_MSG: [u8; 8] = [
        // flags
        0x00, 0x00, 0x00, 0x00,
        // status
        0x01, 0x00, 0x01, 0x00,
    ];

    #[test]
    fn decoding() {
        let msg = NowTerminateMsg::decode(&TERMINATE_MSG).unwrap();
        assert_eq!(msg.flags, 0);
        let status = msg.status;
        assert_eq!(status.severity(), SeverityLevel::Info);
        assert_eq!(status.status_type(), StatusType::Disconnect);
        assert_eq!(status.code(), DisconnectStatusCode::ByLocalUser);
    }

    #[test]
    fn encoding() {
        let status = NowStatus::builder(DisconnectStatusCode::ByLocalUser)
            .severity(SeverityLevel::Info)
            .status_type(StatusType::Disconnect)
            .build();
        let msg = NowTerminateMsg::new(status);
        assert_eq!(msg.encode().unwrap(), TERMINATE_MSG.to_vec());
    }
}
