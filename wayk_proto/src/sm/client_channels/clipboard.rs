use crate::error::ProtoErrorKind;
use crate::message::{
    ChannelName, ClipboardControlState, ClipboardResponseFlags, NowClipboardCapabilitiesReqMsg,
    NowClipboardControlReqMsg, NowClipboardControlRspMsg, NowClipboardFormatDataReqMsg, NowClipboardFormatDataRspMsg,
    NowClipboardFormatListReqMsg, NowClipboardFormatListRspMsg, NowClipboardMsg, NowClipboardResumeReqMsg,
    NowClipboardResumeRspMsg, NowClipboardSuspendReqMsg, NowClipboardSuspendRspMsg, NowVirtualChannel,
};
use crate::sm::{ChannelResponses, ProtoState, SMData, SMEvent, SMEvents, VirtualChannelSM};

pub trait ClipboardChannelCallbackTrait {
    fn on_control_rsp(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        msg: &NowClipboardControlRspMsg,
    ) {
        #![allow(unused_variables)]
    }

    /// Returns true to accept resume request
    fn accept_resume(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        msg: &NowClipboardResumeReqMsg,
    ) -> bool {
        #![allow(unused_variables)]
        true
    }

    fn on_resume_rsp(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        msg: &NowClipboardResumeRspMsg,
    ) {
        #![allow(unused_variables)]
    }

    /// return true to accept suspend request
    fn on_suspend_req(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        msg: &NowClipboardSuspendReqMsg,
    ) -> bool {
        #![allow(unused_variables)]
        true
    }

    fn on_suspend_rsp(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        msg: &NowClipboardSuspendRspMsg,
    ) {
        #![allow(unused_variables)]
    }

    /// Returns true to transfer ownership to peer and false to refuse
    fn transfer_ownership_to_peer(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        msg: &NowClipboardFormatListReqMsg,
    ) -> bool {
        #![allow(unused_variables)]
        true
    }

    fn on_format_list_rsp(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        msg: &NowClipboardFormatListRspMsg,
    ) {
        #![allow(unused_variables)]
    }

    fn on_format_data_req(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        msg: &NowClipboardFormatDataReqMsg,
    ) {
        #![allow(unused_variables)]
    }

    fn on_format_data_rsp(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        msg: &NowClipboardFormatDataRspMsg,
    ) {
        #![allow(unused_variables)]
    }

    /// On clipboard format list req message received if auto fetch is enabled
    fn on_auto_fetch(
        &mut self,
        clipboard_data: &mut ClipboardData,
        sm_data: &mut SMData,
        to_send: &mut ChannelResponses<'_>,
        msg: &NowClipboardFormatListReqMsg,
    ) {
        #![allow(unused_variables)]
    }
}

sa::assert_obj_safe!(ClipboardChannelCallbackTrait);

pub struct DummyClipboardChannelCallback;

impl ClipboardChannelCallbackTrait for DummyClipboardChannelCallback {}

#[derive(PartialEq, Debug, Clone, Copy)]
enum ClipboardState {
    Initial,
    Capabilities,
    Disabled,
    Enabled,
    Terminated,
}

impl ProtoState for ClipboardState {}

#[derive(Debug, Clone, PartialEq)]
pub struct ClipboardData {
    is_owner: bool,
    auto_fetch: bool,
    sequence_id: u16,
}

impl Default for ClipboardData {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardData {
    pub fn new() -> Self {
        Self {
            is_owner: false,
            auto_fetch: true,
            sequence_id: 0,
        }
    }

    pub fn is_owner(&self) -> bool {
        self.is_owner
    }

    pub fn is_auto_fetch_mode(&self) -> bool {
        self.auto_fetch
    }

    pub fn set_auto_fetch(&mut self, auto_fetch: bool) {
        self.auto_fetch = auto_fetch;
    }

    pub fn current_sequence_id(&self) -> u16 {
        self.sequence_id
    }

    pub fn next_sequence_id(&mut self) -> u16 {
        self.sequence_id += 1;
        self.sequence_id
    }
}

pub struct ClipboardChannelSM<UserCallback> {
    state: ClipboardState,
    data: ClipboardData,
    user_callback: UserCallback,
}

impl<UserCallback> ClipboardChannelSM<UserCallback>
where
    UserCallback: ClipboardChannelCallbackTrait,
{
    pub fn new(data: ClipboardData, user_callback: UserCallback) -> Self {
        Self {
            state: ClipboardState::Initial,
            data,
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
        events.push(SMEvent::warn(
            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
            format!(
                "received an unexpected message in state {:?}: {:?}",
                self.state, unexpected
            ),
        ))
    }

    fn h_transition_state(&mut self, events: &mut SMEvents<'_>, state: ClipboardState) {
        self.state = state;
        events.push(SMEvent::transition(state));
    }
}

impl<UserCallback> VirtualChannelSM for ClipboardChannelSM<UserCallback>
where
    UserCallback: ClipboardChannelCallbackTrait,
{
    fn get_channel_name(&self) -> ChannelName {
        ChannelName::Clipboard
    }

    fn is_terminated(&self) -> bool {
        self.state == ClipboardState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        match self.state {
            ClipboardState::Initial => false,
            ClipboardState::Capabilities => true,
            ClipboardState::Disabled => true,
            ClipboardState::Enabled => true,
            ClipboardState::Terminated => false,
        }
    }

    fn update_without_chan_msg<'msg>(
        &mut self,
        _: &mut SMData,
        events: &mut SMEvents<'msg>,
        to_send: &mut ChannelResponses<'msg>,
    ) {
        match self.state {
            ClipboardState::Initial => {
                self.h_transition_state(events, ClipboardState::Capabilities);
                to_send.push(NowClipboardCapabilitiesReqMsg::default());
            }
            _ => {
                self.h_unexpected_without_call(events);
            }
        }
    }

    fn update_with_chan_msg<'msg: 'a, 'a>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        to_send: &mut ChannelResponses<'msg>,
        msg: &'a NowVirtualChannel<'msg>,
    ) {
        let m = if let NowVirtualChannel::Clipboard(m) = msg {
            m
        } else {
            self.h_unexpected_message(events, msg);
            return;
        };

        match self.state {
            ClipboardState::Capabilities => match m {
                NowClipboardMsg::CapabilitiesRsp(m) => {
                    if m.flags.failure() {
                        events.push(SMEvent::error(
                            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
                            "capabilities exchange failed (failure flag received)",
                        ));
                        return;
                    }

                    self.h_transition_state(events, ClipboardState::Disabled);
                    to_send.push(NowClipboardControlReqMsg::new(ClipboardControlState::Auto));
                }
                _ => {
                    self.h_unexpected_message(events, msg);
                }
            },
            ClipboardState::Disabled => match m {
                NowClipboardMsg::ControlRsp(m) => {
                    if m.flags.failure() {
                        events.push(SMEvent::error(
                            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
                            "control setting failed (failure flag received)",
                        ));
                        return;
                    }

                    self.h_transition_state(events, ClipboardState::Enabled);
                    log::trace!("enabled (control: {:?})", m.control_state);
                    self.user_callback.on_control_rsp(&mut self.data, data, to_send, m);
                }
                NowClipboardMsg::ResumeReq(m) => {
                    log::trace!("peer asked for resuming");
                    if self.user_callback.accept_resume(&mut self.data, data, m) {
                        log::trace!("resume request accepted");
                        self.h_transition_state(events, ClipboardState::Enabled);
                        to_send.push(NowClipboardResumeRspMsg::default());
                    } else {
                        log::trace!("resume request refused");
                        to_send.push(NowClipboardResumeRspMsg::new_with_flags(
                            ClipboardResponseFlags::new_empty().set_failure(),
                        ));
                    }
                }
                NowClipboardMsg::ResumeRsp(m) => {
                    if m.flags.failure() {
                        events.push(SMEvent::error(
                            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
                            "resume failed (failure flag received)",
                        ));
                        return;
                    }

                    self.h_transition_state(events, ClipboardState::Enabled);
                    log::trace!("resumed");
                    self.user_callback.on_resume_rsp(&mut self.data, data, to_send, m);
                }
                _ => {
                    self.h_unexpected_message(events, msg);
                }
            },
            ClipboardState::Enabled => match m {
                NowClipboardMsg::SuspendRsp(m) => {
                    if m.flags.failure() {
                        events.push(SMEvent::error(
                            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
                            "suspend failed (failure flag received)",
                        ));
                        return;
                    }

                    self.h_transition_state(events, ClipboardState::Disabled);
                    log::trace!("disabled");
                    self.user_callback.on_suspend_rsp(&mut self.data, data, to_send, m);
                }
                NowClipboardMsg::FormatListReq(m) => {
                    log::trace!("peer asked for ownership");
                    if self.user_callback.transfer_ownership_to_peer(&mut self.data, data, m) {
                        self.data.is_owner = false;
                        log::trace!("ownership transferred to peer");
                        to_send.push(NowClipboardFormatListRspMsg::new(self.data.next_sequence_id()));
                        self.user_callback.on_auto_fetch(&mut self.data, data, to_send, m);
                    } else {
                        log::trace!("ownership transfer refused");
                        to_send.push(NowClipboardFormatListRspMsg::new_with_flags(
                            self.data.next_sequence_id(),
                            ClipboardResponseFlags::new_empty().set_failure(),
                        ));
                    }
                }
                NowClipboardMsg::FormatListRsp(m) => {
                    if m.flags.failure() {
                        events.push(SMEvent::error(
                            ProtoErrorKind::VirtualChannel(self.get_channel_name()),
                            "couldn't take ownership (refused by peer) (failure flag received)",
                        ));
                        return;
                    }

                    self.data.is_owner = true;
                    log::trace!("took ownership");
                    self.user_callback.on_format_list_rsp(&mut self.data, data, to_send, m);
                }
                NowClipboardMsg::FormatDataReq(m) => {
                    if self.data.is_owner || self.data.auto_fetch {
                        self.user_callback.on_format_data_req(&mut self.data, data, to_send, m);
                    } else {
                        events.push(SMEvent::warn(
                            ProtoErrorKind::VirtualChannel(ChannelName::Clipboard),
                            "received format data request while not owner and auto fetch mode is not activated",
                        ))
                    }
                }
                NowClipboardMsg::FormatDataRsp(m) => {
                    if self.data.is_owner {
                        events.push(SMEvent::warn(
                            ProtoErrorKind::VirtualChannel(ChannelName::Clipboard),
                            "received format data response while owner",
                        ));
                    } else {
                        self.user_callback.on_format_data_rsp(&mut self.data, data, to_send, m);
                    }
                }
                _ => {
                    self.h_unexpected_message(events, msg);
                }
            },
            _ => {
                self.h_unexpected_with_call(events);
            }
        }
    }
}
