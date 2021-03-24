use crate::alloc::borrow::ToOwned;
use crate::error::{ProtoError, ProtoErrorKind, ProtoErrorResultExt};
use crate::message::{
    ChannelName, ChatCapabilitiesFlags, NowChatMsg, NowChatSyncMsg, NowChatTextMsg, NowString65535, NowVirtualChannel,
};
use crate::sm::{VirtChannelSMResult, VirtualChannelSM};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use core::str::FromStr;

pub type ChatDataRc = Rc<RefCell<ChatData>>;
pub type TimestampFn = Box<dyn FnMut() -> u32>;

pub trait ChatChannelCallbackTrait {
    fn on_message<'msg>(&mut self, text_msg: &NowChatTextMsg) -> VirtChannelSMResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }

    fn on_synced<'msg>(&mut self) -> VirtChannelSMResult<'msg> {
        Ok(None)
    }
}

sa::assert_obj_safe!(ChatChannelCallbackTrait);

pub struct DummyChatChannelCallback;
impl ChatChannelCallbackTrait for DummyChatChannelCallback {}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatData {
    pub friendly_name: String,
    pub status_text: String,

    pub distant_friendly_name: String,
    pub distant_status_text: String,

    pub capabilities: ChatCapabilitiesFlags,
}

impl Default for ChatData {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatData {
    pub fn new() -> Self {
        Self {
            friendly_name: "Anonymous".to_owned(),
            status_text: "None".to_owned(),
            distant_friendly_name: "Unknown".to_owned(),
            distant_status_text: "None".to_owned(),
            capabilities: ChatCapabilitiesFlags::new_empty(),
        }
    }

    pub fn capabilities(self, capabilities: ChatCapabilitiesFlags) -> Self {
        Self { capabilities, ..self }
    }

    pub fn friendly_name<S: Into<String>>(self, friendly_name: S) -> Self {
        Self {
            friendly_name: friendly_name.into(),
            ..self
        }
    }

    pub fn status_text<S: Into<String>>(self, status_text: S) -> Self {
        Self {
            status_text: status_text.into(),
            ..self
        }
    }

    pub fn into_rc(self) -> ChatDataRc {
        Rc::new(RefCell::new(self))
    }
}

#[derive(PartialEq, Debug)]
enum ChatState {
    Initial,
    Sync,
    Active,
    Terminated,
}

pub struct ChatChannelSM<UserCallback> {
    state: ChatState,
    data: ChatDataRc,
    timestamp_fn: TimestampFn,
    user_callback: UserCallback,
}

impl<UserCallback> ChatChannelSM<UserCallback>
where
    UserCallback: ChatChannelCallbackTrait,
{
    pub fn new(config: ChatDataRc, timestamp_fn: TimestampFn, user_callback: UserCallback) -> Self {
        Self {
            state: ChatState::Initial,
            data: Rc::clone(&config),
            timestamp_fn,
            user_callback,
        }
    }

    fn __unexpected_with_call<'msg>(&self) -> VirtChannelSMResult<'msg> {
        ProtoError::new(ProtoErrorKind::VirtualChannel(self.get_channel_name())).or_desc(format!(
            "unexpected call to `update_with_chan_msg` in state {:?}",
            self.state
        ))
    }

    fn __unexpected_without_call<'msg>(&self) -> VirtChannelSMResult<'msg> {
        ProtoError::new(ProtoErrorKind::VirtualChannel(self.get_channel_name())).or_desc(format!(
            "unexpected call to `update_without_chan_msg` in state {:?}",
            self.state
        ))
    }

    fn __unexpected_message<'msg: 'a, 'a>(&self, unexpected: &'a NowVirtualChannel<'msg>) -> VirtChannelSMResult<'msg> {
        ProtoError::new(ProtoErrorKind::VirtualChannel(self.get_channel_name())).or_desc(format!(
            "received an unexpected message in state {:?}: {:?}",
            self.state, unexpected
        ))
    }
}

impl<UserCallback> VirtualChannelSM for ChatChannelSM<UserCallback>
where
    UserCallback: ChatChannelCallbackTrait,
{
    fn get_channel_name(&self) -> ChannelName {
        ChannelName::Chat
    }

    fn is_terminated(&self) -> bool {
        self.state == ChatState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == ChatState::Active || self.state == ChatState::Sync
    }

    fn update_without_chan_msg<'msg>(&mut self) -> VirtChannelSMResult<'msg> {
        match self.state {
            ChatState::Initial => {
                log::trace!("start syncing");
                self.state = ChatState::Sync;
                Ok(Some(
                    NowChatSyncMsg::new(
                        (self.timestamp_fn)(),
                        self.data.borrow().capabilities,
                        NowString65535::from_str(&self.data.borrow().friendly_name)?,
                    )
                    .status_text(NowString65535::from_str(&self.data.borrow().status_text)?)
                    .into(),
                ))
            }
            _ => self.__unexpected_without_call(),
        }
    }

    fn update_with_chan_msg<'msg: 'a, 'a>(
        &mut self,
        chan_msg: &'a NowVirtualChannel<'msg>,
    ) -> VirtChannelSMResult<'msg> {
        match chan_msg {
            NowVirtualChannel::Chat(msg) => match self.state {
                ChatState::Sync => match msg {
                    NowChatMsg::Sync(msg) => {
                        // update config
                        let mut config_mut = self.data.borrow_mut();
                        config_mut.capabilities.value &= msg.capabilities.value;
                        config_mut.distant_friendly_name = msg.friendly_name.as_str().to_owned();
                        config_mut.distant_status_text = msg.status_text.as_str().to_owned();
                        drop(config_mut);

                        log::trace!("channel synced");
                        self.state = ChatState::Active;
                        self.user_callback.on_synced()
                    }
                    _ => self.__unexpected_message(chan_msg),
                },
                ChatState::Active => match msg {
                    NowChatMsg::Text(msg) => self.user_callback.on_message(msg),
                    _ => self.__unexpected_message(chan_msg),
                },
                _ => self.__unexpected_with_call(),
            },
            _ => self.__unexpected_message(chan_msg),
        }
    }
}
