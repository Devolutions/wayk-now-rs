use crate::alloc::borrow::ToOwned;
use crate::error::ProtoErrorKind;
use crate::message::{
    ChannelName, ChatCapabilitiesFlags, NowChatMsg, NowChatSyncMsg, NowChatTextMsg, NowString65535, NowVirtualChannel,
};
use crate::sm::{ChannelResponses, ProtoState, SMData, SMEvent, SMEvents, VirtualChannelSM};
use alloc::boxed::Box;
use alloc::string::String;
use core::str::FromStr;

pub type TimestampFn = Box<dyn FnMut() -> u32>;

pub trait ChatChannelCallbackTrait {
    fn on_message(&mut self, chat_data: &mut ChatData, to_send: &mut ChannelResponses<'_>, text_msg: &NowChatTextMsg) {
        #![allow(unused_variables)]
    }

    fn on_synced(&mut self, chat_data: &mut ChatData, to_send: &mut ChannelResponses<'_>) {
        #![allow(unused_variables)]
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
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum ChatState {
    Initial,
    Sync,
    Active,
    Terminated,
}

impl ProtoState for ChatState {}

pub struct ChatChannelSM<UserCallback> {
    state: ChatState,
    data: ChatData,
    timestamp_fn: TimestampFn,
    user_callback: UserCallback,
}

impl<UserCallback> ChatChannelSM<UserCallback>
where
    UserCallback: ChatChannelCallbackTrait,
{
    pub fn new(config: ChatData, timestamp_fn: TimestampFn, user_callback: UserCallback) -> Self {
        Self {
            state: ChatState::Initial,
            data: config,
            timestamp_fn,
            user_callback,
        }
    }

    fn h_unexpected_with_call<'msg>(&self, events: &mut SMEvents<'msg>) {
        events.push(SMEvent::error(
            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
            format!("unexpected call to `update_with_chan_msg` in state {:?}", self.state),
        ))
    }

    fn h_unexpected_without_call<'msg>(&self, events: &mut SMEvents<'msg>) {
        events.push(SMEvent::error(
            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
            format!("unexpected call to `update_without_chan_msg` in state {:?}", self.state),
        ))
    }

    fn h_unexpected_message<'msg: 'a, 'a>(&self, events: &mut SMEvents<'msg>, unexpected: &'a NowVirtualChannel<'msg>) {
        events.push(SMEvent::error(
            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
            format!(
                "received an unexpected message in state {:?}: {:?}",
                self.state, unexpected,
            ),
        ))
    }

    fn h_transition_state(&mut self, events: &mut SMEvents<'_>, state: ChatState) {
        self.state = state;
        events.push(SMEvent::transition(state));
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

    fn update_without_chan_msg<'msg>(
        &mut self,
        _: &mut SMData,
        events: &mut SMEvents<'msg>,
        to_send: &mut ChannelResponses<'msg>,
    ) {
        match self.state {
            ChatState::Initial => {
                log::trace!("start syncing");

                let friendly_name = match NowString65535::from_str(&self.data.friendly_name) {
                    Ok(s) => s,
                    Err(e) => {
                        events.push(SMEvent::Error(e));
                        return;
                    }
                };

                let status_text = match NowString65535::from_str(&self.data.status_text) {
                    Ok(s) => s,
                    Err(e) => {
                        events.push(SMEvent::Error(e));
                        return;
                    }
                };

                to_send.push(
                    NowChatSyncMsg::new((self.timestamp_fn)(), self.data.capabilities, friendly_name)
                        .status_text(status_text),
                );

                self.h_transition_state(events, ChatState::Sync);
            }
            _ => self.h_unexpected_without_call(events),
        }
    }

    fn update_with_chan_msg<'msg: 'a, 'a>(
        &mut self,
        _: &mut SMData,
        events: &mut SMEvents<'msg>,
        to_send: &mut ChannelResponses<'msg>,
        chan_msg: &'a NowVirtualChannel<'msg>,
    ) {
        match chan_msg {
            NowVirtualChannel::Chat(msg) => match self.state {
                ChatState::Sync => match msg {
                    NowChatMsg::Sync(msg) => {
                        // update config
                        self.data.capabilities.value &= msg.capabilities.value;
                        self.data.distant_friendly_name = msg.friendly_name.as_str().to_owned();
                        self.data.distant_status_text = msg.status_text.as_str().to_owned();

                        log::trace!("channel synced");
                        self.state = ChatState::Active;
                        self.user_callback.on_synced(&mut self.data, to_send);
                    }
                    _ => self.h_unexpected_message(events, chan_msg),
                },
                ChatState::Active => match msg {
                    NowChatMsg::Text(msg) => self.user_callback.on_message(&mut self.data, to_send, msg),
                    _ => self.h_unexpected_message(events, chan_msg),
                },
                _ => self.h_unexpected_with_call(events),
            },
            _ => self.h_unexpected_message(events, chan_msg),
        }
    }
}
