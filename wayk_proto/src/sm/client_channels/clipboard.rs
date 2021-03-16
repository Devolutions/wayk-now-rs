use crate::error::{ProtoError, ProtoErrorKind, ProtoErrorResultExt};
use crate::message::{
    ChannelName, ClipboardControlState, ClipboardResponseFlags, NowClipboardCapabilitiesReqMsg,
    NowClipboardControlReqMsg, NowClipboardControlRspMsg, NowClipboardFormatDataReqMsg, NowClipboardFormatDataRspMsg,
    NowClipboardFormatListReqMsg, NowClipboardFormatListRspMsg, NowClipboardMsg, NowClipboardResumeReqMsg,
    NowClipboardResumeRspMsg, NowClipboardSuspendReqMsg, NowClipboardSuspendRspMsg, NowVirtualChannel,
};
use crate::sm::{VirtChannelSMResult, VirtualChannelSM};
use std::cell::RefCell;
use std::rc::Rc;

pub type ClipboardDataRc = Rc<RefCell<ClipboardData>>;

pub trait ClipboardChannelCallbackTrait {
    fn on_control_rsp<'msg>(&mut self, msg: &NowClipboardControlRspMsg) -> VirtChannelSMResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }

    /// return true to accept resume request
    fn on_resume_req(&mut self, msg: &NowClipboardResumeReqMsg) -> bool {
        #![allow(unused_variables)]
        true
    }

    fn on_resume_rsp<'msg>(&mut self, msg: &NowClipboardResumeRspMsg) -> VirtChannelSMResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }

    /// return true to accept suspend request
    fn on_suspend_req(&mut self, msg: &NowClipboardSuspendReqMsg) -> bool {
        #![allow(unused_variables)]
        true
    }

    fn on_suspend_rsp<'msg>(&mut self, msg: &NowClipboardSuspendRspMsg) -> VirtChannelSMResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }

    /// return true to transfer ownership to peer and false to refuse
    fn on_format_list_req(&mut self, msg: &NowClipboardFormatListReqMsg) -> bool {
        #![allow(unused_variables)]
        true
    }

    fn on_format_list_rsp<'msg>(&mut self, msg: &NowClipboardFormatListRspMsg) -> VirtChannelSMResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }

    fn on_format_data_req<'msg>(&mut self, msg: &NowClipboardFormatDataReqMsg) -> VirtChannelSMResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }

    fn on_format_data_rsp<'msg>(&mut self, msg: &NowClipboardFormatDataRspMsg) -> VirtChannelSMResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }

    fn auto_fetch_data<'msg>(&mut self) -> VirtChannelSMResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }
}

sa::assert_obj_safe!(ClipboardChannelCallbackTrait);

pub struct DummyClipboardChannelCallback;
impl ClipboardChannelCallbackTrait for DummyClipboardChannelCallback {}

#[derive(PartialEq, Debug)]
enum ClipboardState {
    Initial,
    Capabilities,
    Disabled,
    Enabled,
    AutoFetch,
    Terminated,
}

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

    pub fn into_rc(self) -> ClipboardDataRc {
        Rc::new(RefCell::new(self))
    }
}

pub struct ClipboardChannelSM<UserCallback> {
    state: ClipboardState,
    data: ClipboardDataRc,
    user_callback: UserCallback,
}

impl<UserCallback> ClipboardChannelSM<UserCallback>
where
    UserCallback: ClipboardChannelCallbackTrait,
{
    pub fn new(data: ClipboardDataRc, user_callback: UserCallback) -> Self {
        Self {
            state: ClipboardState::Initial,
            data,
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

    fn __check_failure<'msg>(&mut self, flags: ClipboardResponseFlags) -> VirtChannelSMResult<'msg> {
        if flags.failure() {
            ProtoError::new(ProtoErrorKind::VirtualChannel(self.get_channel_name()))
        } else {
            Ok(None)
        }
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
            ClipboardState::AutoFetch => false,
            ClipboardState::Terminated => false,
        }
    }

    fn update_without_chan_msg<'msg>(&mut self) -> VirtChannelSMResult<'msg> {
        match self.state {
            ClipboardState::Initial => {
                log::trace!("start");
                self.state = ClipboardState::Capabilities;
                Ok(Some(NowClipboardCapabilitiesReqMsg::default().into()))
            }
            ClipboardState::AutoFetch => {
                self.state = ClipboardState::Enabled;
                self.user_callback.auto_fetch_data()
            }
            _ => self.__unexpected_without_call(),
        }
    }

    fn update_with_chan_msg<'msg: 'a, 'a>(
        &mut self,
        chan_msg: &'a NowVirtualChannel<'msg>,
    ) -> VirtChannelSMResult<'msg> {
        match chan_msg {
            NowVirtualChannel::Clipboard(msg) => match self.state {
                ClipboardState::Capabilities => match msg {
                    NowClipboardMsg::CapabilitiesRsp(msg) => {
                        self.__check_failure(msg.flags)
                            .or_desc("capabilities exchange failed")?;
                        self.state = ClipboardState::Disabled;
                        log::trace!("capabilities exchange succeeded");
                        Ok(Some(NowClipboardControlReqMsg::new(ClipboardControlState::Auto).into()))
                    }
                    _ => self.__unexpected_message(chan_msg),
                },
                ClipboardState::Disabled => match msg {
                    NowClipboardMsg::ControlRsp(msg) => {
                        self.__check_failure(msg.flags).or_desc("control setting failed")?;
                        self.state = ClipboardState::Enabled;
                        log::trace!("enabled (control: {:?})", msg.control_state);
                        self.user_callback.on_control_rsp(msg)
                    }
                    NowClipboardMsg::ResumeReq(msg) => {
                        log::trace!("peer asked for resuming");
                        if self.user_callback.on_resume_req(msg) {
                            log::trace!("resume request accepted");
                            self.state = ClipboardState::Enabled;
                            Ok(Some(NowClipboardResumeRspMsg::default().into()))
                        } else {
                            log::trace!("resume request refused");
                            Ok(Some(
                                NowClipboardResumeRspMsg::new_with_flags(
                                    ClipboardResponseFlags::new_empty().set_failure(),
                                )
                                .into(),
                            ))
                        }
                    }
                    NowClipboardMsg::ResumeRsp(msg) => {
                        self.__check_failure(msg.flags).or_desc("resume failed")?;
                        self.state = ClipboardState::Enabled;
                        log::trace!("resumed");
                        self.user_callback.on_resume_rsp(msg)
                    }
                    _ => self.__unexpected_message(chan_msg),
                },
                ClipboardState::Enabled => match msg {
                    NowClipboardMsg::SuspendRsp(msg) => {
                        self.__check_failure(msg.flags).or_desc("suspend failed")?;
                        self.state = ClipboardState::Disabled;
                        log::trace!("disabled");
                        Ok(None)
                    }
                    NowClipboardMsg::FormatListReq(msg) => {
                        log::trace!("peer asked for ownership");
                        if self.user_callback.on_format_list_req(msg) {
                            let mut data_mut = self.data.borrow_mut();
                            data_mut.is_owner = false;
                            log::trace!("ownership transferred to peer");
                            if data_mut.auto_fetch {
                                self.state = ClipboardState::AutoFetch;
                            }
                            Ok(Some(
                                NowClipboardFormatListRspMsg::new(data_mut.next_sequence_id()).into(),
                            ))
                        } else {
                            log::trace!("ownership transfer refused");
                            Ok(Some(
                                NowClipboardFormatListRspMsg::new_with_flags(
                                    self.data.borrow_mut().next_sequence_id(),
                                    ClipboardResponseFlags::new_empty().set_failure(),
                                )
                                .into(),
                            ))
                        }
                    }
                    NowClipboardMsg::FormatListRsp(msg) => {
                        self.__check_failure(msg.flags)
                            .or_desc("couldn't take ownership (refused by peer)")?;
                        self.data.borrow_mut().is_owner = true;
                        log::trace!("took ownership");
                        self.user_callback.on_format_list_rsp(msg)
                    }
                    NowClipboardMsg::FormatDataReq(msg) => {
                        let data = self.data.borrow();
                        if data.is_owner || data.auto_fetch {
                            self.user_callback.on_format_data_req(msg)
                        } else {
                            ProtoError::new(ProtoErrorKind::VirtualChannel(ChannelName::Clipboard)).or_desc(
                                "received format data request while not owner and auto fetch mode is not activated",
                            )
                        }
                    }
                    NowClipboardMsg::FormatDataRsp(msg) => {
                        if self.data.borrow().is_owner {
                            ProtoError::new(ProtoErrorKind::VirtualChannel(ChannelName::Clipboard))
                                .or_desc("received format data response while owner")
                        } else {
                            self.user_callback.on_format_data_rsp(msg)
                        }
                    }
                    _ => self.__unexpected_message(chan_msg),
                },
                _ => self.__unexpected_with_call(),
            },
            _ => self.__unexpected_message(chan_msg),
        }
    }
}
