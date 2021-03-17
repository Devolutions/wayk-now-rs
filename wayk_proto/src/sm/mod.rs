pub mod client_channels;
/** STATE MACHINE **/
pub mod client_connection;

// re-export
pub use client_channels::*;
pub use client_connection::*;

use crate::error::ProtoError;
use crate::message::{AuthType, ChannelName, NowCapset, NowChannelDef, NowMessage, NowVirtualChannel};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

// === connection sequence ===

pub type ConnectionSMSharedDataRc = Rc<RefCell<ConnectionSMSharedData>>;
pub type ConnectionSMResult<'a> = Result<Option<NowMessage<'a>>, ProtoError>;

pub trait ConnectionSM {
    fn set_shared_data(&mut self, shared_data: ConnectionSMSharedDataRc);
    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc>;
    fn is_terminated(&self) -> bool;
    fn waiting_for_packet(&self) -> bool;
    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg>;
    fn update_with_message<'msg: 'a, 'a>(&mut self, message: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg>;
    fn is_running(&self) -> bool {
        !self.is_terminated()
    }
}

sa::assert_obj_safe!(ConnectionSM);

pub struct ConnectionSMSharedData {
    pub available_auth_types: Vec<AuthType>,
    pub capabilities: Vec<NowCapset<'static>>,
    pub channels: Vec<NowChannelDef>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionState {
    Handshake,
    Negotiate,
    Authenticate,
    Associate,
    Capabilities,
    Channels,
    Final,
}

pub trait ConnectionSeqCallbackTrait {
    fn on_handshake_completed(&mut self, shared_data: &ConnectionSMSharedData) {
        #![allow(unused_variables)]
    }

    fn on_negotiate_completed(&mut self, shared_data: &ConnectionSMSharedData) {
        #![allow(unused_variables)]
    }

    fn on_authenticate_completed(&mut self, shared_data: &ConnectionSMSharedData) {
        #![allow(unused_variables)]
    }

    fn on_associate_completed(&mut self, shared_data: &ConnectionSMSharedData) {
        #![allow(unused_variables)]
    }

    fn on_capabilities_completed(&mut self, shared_data: &ConnectionSMSharedData) {
        #![allow(unused_variables)]
    }

    fn on_connection_completed(&mut self, shared_data: &ConnectionSMSharedData) {
        #![allow(unused_variables)]
    }
}

sa::assert_obj_safe!(ConnectionSeqCallbackTrait);

pub struct DummyConnectionSeqCallback;
impl ConnectionSeqCallbackTrait for DummyConnectionSeqCallback {}

pub struct DummyConnectionSM;
impl ConnectionSM for DummyConnectionSM {
    fn set_shared_data(&mut self, _: ConnectionSMSharedDataRc) {
        log::warn!("call to `DummyConnectionSM::set_shared_data`");
    }

    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc> {
        log::warn!("call to `DummyConnectionSM::get_shared_data`");
        None
    }

    fn is_terminated(&self) -> bool {
        log::warn!("call to `DummyConnectionSM::is_terminated`");
        true
    }

    fn waiting_for_packet(&self) -> bool {
        log::warn!("call to `DummyConnectionSM::waiting_for_packet`");
        false
    }

    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg> {
        log::warn!("call to `DummyConnectionSM::update_without_message`");
        Ok(None)
    }

    fn update_with_message<'msg: 'a, 'a>(&mut self, _msg: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg> {
        log::warn!("call to `DummyConnectionSM::update_with_message`");
        Ok(None)
    }
}

// === virtual channels ===

pub type VirtChannelSMResult<'a> = Result<Option<NowVirtualChannel<'a>>, ProtoError>;

pub trait VirtualChannelSM {
    fn get_channel_name(&self) -> ChannelName;
    fn is_terminated(&self) -> bool;
    fn waiting_for_packet(&self) -> bool;
    fn update_without_chan_msg<'msg>(&mut self) -> VirtChannelSMResult<'msg>;
    fn update_with_chan_msg<'msg: 'a, 'a>(
        &mut self,
        chan_msg: &'a NowVirtualChannel<'msg>,
    ) -> VirtChannelSMResult<'msg>;
    fn is_running(&self) -> bool {
        !self.is_terminated()
    }
}

sa::assert_obj_safe!(VirtualChannelSM);

pub struct DummyVirtChannelSM;
impl VirtualChannelSM for DummyVirtChannelSM {
    fn get_channel_name(&self) -> ChannelName {
        log::warn!("call to `DummyVirtChannelSM::get_channel_name`");
        ChannelName::Unknown("Dummy".into())
    }

    fn is_terminated(&self) -> bool {
        log::warn!("call to `DummyVirtChannelSM::is_terminated`");
        true
    }

    fn waiting_for_packet(&self) -> bool {
        log::warn!("call to `DummyVirtChannelSM::waiting_for_packet`");
        false
    }

    fn update_without_chan_msg<'msg>(&mut self) -> VirtChannelSMResult<'msg> {
        log::warn!("call to `DummyVirtChannelSM::update_without_virt_msg`");
        Ok(None)
    }

    fn update_with_chan_msg<'msg: 'a, 'a>(&mut self, _: &'a NowVirtualChannel<'msg>) -> VirtChannelSMResult<'msg> {
        log::warn!("call to `DummyVirtChannelSM::update_with_chan_msg`");
        Ok(None)
    }
}
