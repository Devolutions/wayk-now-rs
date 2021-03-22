pub mod client_channels;
pub mod client_connection;

// re-export
pub use client_channels::*;
pub use client_connection::*;

use crate::error::{ProtoError, ProtoErrorKind};
use crate::message::{AuthType, ChannelName, NowCapset, NowChannelDef, NowMessage, NowVirtualChannel};
use crate::packet::NowPacket;
use crate::sharee::ShareeState;
use alloc::vec::Vec;
use core::fmt::Debug;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

// === State Machine Event == //

pub struct SMEvents<'a>(Vec<SMEvent<'a>>);

impl Default for SMEvents<'_> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'a> SMEvents<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<'event: 'a>(&mut self, event: SMEvent<'event>) {
        self.0.push(event);
    }

    pub fn peek(&self) -> &[SMEvent<'a>] {
        self.0.as_slice()
    }

    pub fn unpack(self) -> Vec<SMEvent<'a>> {
        self.0
    }
}

pub enum SMEvent<'event> {
    StateTransition(Box<dyn ProtoState>),
    PacketToSend(NowPacket<'event>),
    Data(Box<dyn ProtoData>),
    Warn(ProtoError),
    Error(ProtoError),
    Fatal(ProtoError),
}

impl<'event> SMEvent<'event> {
    pub fn transition(s: impl ProtoState) -> Self {
        Self::StateTransition(Box::new(s))
    }

    pub fn data(v: impl ProtoData) -> Self {
        Self::Data(Box::new(v))
    }

    pub fn warn(kind: ProtoErrorKind, s: impl Into<alloc::borrow::Cow<'static, str>>) -> Self {
        Self::Warn(ProtoError::new(kind).with_desc(s))
    }

    pub fn error(kind: ProtoErrorKind, s: impl Into<alloc::borrow::Cow<'static, str>>) -> Self {
        Self::Error(ProtoError::new(kind).with_desc(s))
    }

    pub fn fatal(kind: ProtoErrorKind, s: impl Into<alloc::borrow::Cow<'static, str>>) -> Self {
        Self::Fatal(ProtoError::new(kind).with_desc(s))
    }
}

pub trait ProtoState: Any + Debug {}

pub trait ProtoData: Any + Debug {}

// === State Machine Data === //

#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

pub struct SMData {
    pub supported_auths: Vec<AuthType>,
    pub capabilities: Vec<NowCapset<'static>>,
    pub channel_defs: Vec<NowChannelDef>,
    extra: HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>,
}

impl SMData {
    #[inline]
    pub fn new(
        supported_auths: Vec<AuthType>,
        capabilities: Vec<NowCapset<'static>>,
        channel_defs: Vec<NowChannelDef>,
    ) -> Self {
        Self {
            supported_auths,
            capabilities,
            channel_defs,
            extra: HashMap::default(),
        }
    }

    pub fn extra_insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.extra
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| (boxed as Box<dyn Any + 'static>).downcast().ok().map(|boxed| *boxed))
    }

    pub fn extra_get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extra
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
    }

    pub fn extra_get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.extra
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut())
    }

    pub fn extra_remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.extra
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| (boxed as Box<dyn Any + 'static>).downcast().ok().map(|boxed| *boxed))
    }

    #[inline]
    pub fn extra_clear(&mut self) {
        self.extra.clear();
    }
}

// === connection sequence === //

pub type ConnectionSMResult<'a> = Result<Option<NowMessage<'a>>, ProtoError>;

pub trait ConnectionSM {
    fn is_terminated(&self) -> bool;

    fn waiting_for_packet(&self) -> bool;

    fn update_without_message<'msg>(&mut self, data: &mut SMData, events: &mut SMEvents<'msg>);

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        msg: &'a NowMessage<'msg>,
    );

    fn is_running(&self) -> bool {
        !self.is_terminated()
    }
}

pub struct DummyConnectionSM;

impl ConnectionSM for DummyConnectionSM {
    fn is_terminated(&self) -> bool {
        true
    }

    fn waiting_for_packet(&self) -> bool {
        false
    }

    fn update_without_message<'msg>(&mut self, _: &mut SMData, events: &mut SMEvents<'msg>) {
        events.push(SMEvent::warn(
            ProtoErrorKind::Sharee(ShareeState::Connection),
            "call to `DummyConnectionSM::update_without_message`",
        ))
    }

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        _: &mut SMData,
        events: &mut SMEvents<'msg>,
        _: &'a NowMessage<'msg>,
    ) {
        events.push(SMEvent::warn(
            ProtoErrorKind::Sharee(ShareeState::Connection),
            "call to `DummyConnectionSM::update_with_message`",
        ))
    }
}

sa::assert_obj_safe!(ConnectionSM);

// === virtual channels === //

pub struct ChannelResponses<'a> {
    inner: Vec<(ChannelName, NowVirtualChannel<'a>)>,
    current_channel_name: ChannelName,
}

impl Default for ChannelResponses<'_> {
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            current_channel_name: ChannelName::Unknown("unbound".into()),
        }
    }
}

impl<'a> ChannelResponses<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_current_channel_name(&mut self, name: ChannelName) {
        self.current_channel_name = name;
    }

    pub fn push<'msg: 'a>(&mut self, msg: impl Into<NowVirtualChannel<'msg>>) {
        self.inner.push((self.current_channel_name.clone(), msg.into()));
    }

    pub fn peek(&self) -> &[(ChannelName, NowVirtualChannel<'a>)] {
        self.inner.as_slice()
    }

    pub fn unpack(self) -> Vec<(ChannelName, NowVirtualChannel<'a>)> {
        self.inner
    }
}

pub trait VirtualChannelSM {
    fn get_channel_name(&self) -> ChannelName;

    fn is_terminated(&self) -> bool;

    fn waiting_for_packet(&self) -> bool;

    fn update_without_chan_msg<'msg>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        to_send: &mut ChannelResponses<'msg>,
    );

    fn update_with_chan_msg<'msg: 'a, 'a>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        to_send: &mut ChannelResponses<'msg>,
        msg: &'a NowVirtualChannel<'msg>,
    );

    fn is_running(&self) -> bool {
        !self.is_terminated()
    }
}

sa::assert_obj_safe!(VirtualChannelSM);
