use crate::message::{AccessControlCode, AccessFlags};

__flags_struct! {
    AccessControlFlags: u8 => {
        failure = FAILURE = 0x80,
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub enum AccessControlMessageType {
    #[value = 0x01]
    Req,
    #[value = 0x02]
    Rsp,
    #[value = 0x03]
    Ntf,
    #[fallback]
    Other(u8),
}

__flags_struct! {
    AccessReason: u16 => {
        denied = DENIED = 0x0001,
        timeout = TIMEOUT = 0x0002,
        disabled = DISABLED = 0x0003,
    }
}

// NOW_ACCESS_CONTROL_REQ_MSG

#[derive(Debug, Clone, Encode, Decode)]
pub struct NowAcessControlReq {
    subtype: AccessControlMessageType,
    flags: AccessControlFlags,
    pub id: AccessControlCode,
    pub timeout: u16,
}

impl NowAcessControlReq {
    pub const SUBTYPE: AccessControlMessageType = AccessControlMessageType::Req;

    pub fn new(id: AccessControlCode, timeout: u16) -> Self {
        Self {
            subtype: Self::SUBTYPE,
            flags: AccessControlFlags::new_empty(),
            id,
            timeout,
        }
    }
}

// NOW_ACCESS_CONTROL_RSP_MSG

#[derive(Debug, Clone, Encode, Decode)]
pub struct NowAcessControlRsp {
    subtype: AccessControlMessageType,
    pub flags: AccessControlFlags,
    pub id: AccessControlCode,
    pub reason: AccessReason,
}

// NOW_ACCESS_CONTROL_NTF_MSG

#[derive(Debug, Clone, Encode, Decode)]
pub struct NowAcessControlNtf {
    subtype: AccessControlMessageType,
    pub flags: AccessControlFlags,
    pub id: AccessControlCode,
    pub status: AccessFlags,
}

// NOW_ACCESS_MSG

#[derive(Debug, Clone, Encode, Decode)]
#[meta_enum = "AccessControlMessageType"]
pub enum NowAccessMsg {
    Req(NowAcessControlReq),
    Rsp(NowAcessControlRsp),
    Ntf(NowAcessControlNtf),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::{Decode, Encode};

    const ACCESS_CONTROL_REQ_MSG: [u8; 6] = [0x01, 0x00, 0x06, 0x00, 0x1e, 0x00];

    #[test]
    fn access_control_req_decoding() {
        let msg = NowAccessMsg::decode(&ACCESS_CONTROL_REQ_MSG).unwrap();

        if let NowAccessMsg::Req(msg) = msg {
            assert_eq!(msg.subtype, AccessControlMessageType::Req);
            assert_eq!(msg.flags, 0x00);
            assert_eq!(msg.id, AccessControlCode::Chat);
            assert_eq!(msg.timeout, 30);
        } else {
            panic!("Expected a request message, found {:?}", msg);
        }
    }

    #[test]
    fn access_control_req_encoding() {
        let req = NowAcessControlReq::new(AccessControlCode::Chat, 30);
        assert_eq!(req.encode().unwrap(), ACCESS_CONTROL_REQ_MSG.to_vec());
    }

    const ACCESS_CONTROL_RSP_MSG: [u8; 6] = [0x02, 0x80, 0x06, 0x00, 0x02, 0x00];

    #[test]
    fn access_control_rsp_decoding() {
        let msg = NowAccessMsg::decode(&ACCESS_CONTROL_RSP_MSG).unwrap();
        if let NowAccessMsg::Rsp(msg) = msg {
            assert_eq!(msg.subtype, AccessControlMessageType::Rsp);
            assert_eq!(msg.flags, AccessControlFlags::FAILURE);
            assert_eq!(msg.id, AccessControlCode::Chat);
            assert_eq!(msg.reason, AccessReason::TIMEOUT);
        } else {
            panic!("Expected a response message, found {:?}", msg);
        }
    }

    const ACCESS_CONTROL_NTF_MSG: [u8; 6] = [0x03, 0x00, 0x03, 0x00, 0x01, 0x00];

    #[test]
    fn access_control_ntf_decoding() {
        let msg = NowAccessMsg::decode(&ACCESS_CONTROL_NTF_MSG).unwrap();
        if let NowAccessMsg::Ntf(msg) = msg {
            assert_eq!(msg.subtype, AccessControlMessageType::Ntf);
            assert_eq!(msg.flags, AccessControlFlags::new_empty());
            assert_eq!(msg.id, AccessControlCode::Clipboard);
            assert_eq!(msg.status, AccessFlags::ALLOWED);
        } else {
            panic!("Expected a ntf message, found {:?}", msg);
        }
    }
}
