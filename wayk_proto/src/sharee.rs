use crate::channels_manager::{ChannelsManager, ChannelsManagerResult};
use crate::error::{ProtoError, ProtoErrorKind, ProtoErrorResultExt};
use crate::message::NowTerminateMsg;
/** SHAREE **/
use crate::message::{NowBody, NowMessage, VirtChannelsCtx};
use crate::packet::NowPacket;
use crate::sm::{ConnectionSM, ConnectionSMResult, ConnectionSMSharedData, ConnectionSMSharedDataRc};

pub type ShareeResult<'a> = Result<Option<NowPacket<'a>>, ProtoError>;

pub trait ShareeCallbackTrait {
    fn on_enter_active_state(&mut self, shared_data: &ConnectionSMSharedData) {
        #![allow(unused_variables)]
    }

    /// called on any message
    fn on_any_message<'msg: 'a, 'a>(&mut self, message: &'a NowMessage<'msg>) {
        #![allow(unused_variables)]
    }

    /// called only on messages unprocessed by Sharee. An answer can be returned.
    fn on_unprocessed_message<'msg: 'a, 'a>(&mut self, message: &'a NowMessage<'msg>) -> ShareeResult<'msg> {
        #![allow(unused_variables)]
        Ok(None)
    }
}

sa::assert_obj_safe!(ShareeCallbackTrait);

pub struct DummyShareeCallback;
impl ShareeCallbackTrait for DummyShareeCallback {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShareeState {
    Connection,
    Active,
    Final,
}

pub struct Sharee<ConnectionSeq, UserCallback> {
    state: ShareeState,
    connection_seq: ConnectionSeq,
    channels_manager: ChannelsManager,
    user_callback: UserCallback,
    shared_data: ConnectionSMSharedDataRc,
    channels_ctx: VirtChannelsCtx,
}

impl<ConnectionSeq, UserCallback> Sharee<ConnectionSeq, UserCallback>
where
    ConnectionSeq: ConnectionSM,
    UserCallback: ShareeCallbackTrait,
{
    pub fn new(connection_sm: ConnectionSeq, channels_manager: ChannelsManager, user_callback: UserCallback) -> Self {
        let shared_data = connection_sm
            .get_shared_data()
            .expect("couldn't retrieve shared data from connection sequence state machine"); // should never panic
        Self {
            state: ShareeState::Connection,
            connection_seq: connection_sm,
            channels_manager,
            user_callback,
            shared_data,
            channels_ctx: VirtChannelsCtx::new(),
        }
    }

    pub fn get_state(&self) -> ShareeState {
        self.state
    }

    pub fn get_connection_seq(&self) -> &ConnectionSeq {
        &self.connection_seq
    }

    pub fn is_terminated(&self) -> bool {
        self.state == ShareeState::Final
    }

    pub fn is_running(&self) -> bool {
        !self.is_terminated()
    }

    pub fn waiting_for_packet(&self) -> bool {
        match self.state {
            ShareeState::Connection => self.connection_seq.waiting_for_packet(),
            ShareeState::Active => self.channels_manager.waiting_for_packet(),
            ShareeState::Final => false,
        }
    }

    pub fn update_without_body<'msg>(&mut self) -> ShareeResult<'msg> {
        match self.state {
            ShareeState::Connection => {
                let answer = self.connection_seq.update_without_message();
                if self.connection_seq.is_terminated() {
                    self.__go_to_active_state();
                }
                self.__check_result(&answer);
                answer.map(|o| o.map(NowPacket::from))
            }
            ShareeState::Active => {
                let result = self.channels_manager.update_without_virt_msg();
                self.__map_channels_manager_result(result)
            }
            ShareeState::Final => Ok(NowPacket::from_message(NowTerminateMsg::default()).into()),
        }
    }

    pub fn update_with_body<'msg: 'a, 'a>(&mut self, body: &'a NowBody<'msg>) -> ShareeResult<'msg> {
        match body {
            NowBody::Message(msg) => match self.state {
                ShareeState::Connection => {
                    let answer = self.connection_seq.update_with_message(msg);
                    if self.connection_seq.is_terminated() {
                        self.__go_to_active_state();
                    }
                    self.__check_result(&answer);
                    self.user_callback.on_any_message(msg);
                    answer.map(|o| o.map(NowPacket::from))
                }
                ShareeState::Active => match msg {
                    NowMessage::Terminate(_) => {
                        self.state = ShareeState::Final;
                        self.user_callback.on_any_message(msg);
                        Ok(None)
                    }
                    msg => {
                        let answer = self.user_callback.on_unprocessed_message(msg);
                        self.user_callback.on_any_message(msg);
                        answer
                    }
                },
                ShareeState::Final => ProtoError::new(ProtoErrorKind::Sharee(self.state))
                    .or_desc("unexpected call to `Sharee::update_with_body` in final state with a now message"),
            },
            NowBody::VirtualChannel(chan_msg) => match self.state {
                ShareeState::Connection => ProtoError::new(ProtoErrorKind::Sharee(self.state)).or_desc(
                    "unexpected call to `Sharee::update_with_body` in connection state with a virtual channel message",
                ),
                ShareeState::Active => {
                    let result = self.channels_manager.update_with_virt_msg(chan_msg);
                    self.__map_channels_manager_result(result)
                }
                ShareeState::Final => ProtoError::new(ProtoErrorKind::Sharee(self.state)).or_desc(
                    "unexpected call to `Sharee::update_with_body` in final state with a virtual channel message",
                ),
            },
        }
    }

    pub fn get_channels_ctx(&self) -> &VirtChannelsCtx {
        &self.channels_ctx
    }

    fn __check_result(&mut self, result: &ConnectionSMResult<'_>) {
        if result.is_err() {
            log::trace!("an error occurred. Set sharee state to final state.");
            self.state = ShareeState::Final;
        }
    }

    fn __go_to_active_state(&mut self) {
        log::trace!("enter active state.");
        self.state = ShareeState::Active;
        for def in &self.shared_data.borrow().channels {
            self.channels_ctx.insert(def.flags.value as u8, def.name.clone());
        }
        log::debug!("virtual channels context: {:#?}", self.channels_ctx);
        self.user_callback.on_enter_active_state(&self.shared_data.borrow());
    }

    fn __map_channels_manager_result<'msg>(&self, chan_result: ChannelsManagerResult<'msg>) -> ShareeResult<'msg> {
        match chan_result {
            Ok(Some((name, virt_chan))) => Ok(Some(NowPacket::from_virt_channel(
                virt_chan,
                self.channels_ctx
                    .get_id_by_channel(&name)
                    .chain(ProtoErrorKind::Sharee(ShareeState::Active))
                    .or_else_desc(|| format!("channel id for {:?} not found in channels context", name))?,
            ))),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
