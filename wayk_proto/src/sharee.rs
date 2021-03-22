use crate::channels_manager::ChannelsManager;
use crate::error::ProtoErrorKind;
use crate::message::{
    AuthType, ChannelName, NowBody, NowCapset, NowChannelDef, NowMessage, NowTerminateMsg, VirtChannelsCtx,
};
use crate::packet::NowPacket;
use crate::sm::{ChannelResponses, ConnectionSM, ProtoState, SMData, SMEvent, SMEvents};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShareeState {
    Connection,
    Active,
    Final,
}

impl ProtoState for ShareeState {}

pub struct Sharee<ConnectionSeq> {
    state: ShareeState,
    connection_seq: ConnectionSeq,
    channels_manager: ChannelsManager,
    sm_data: SMData,
    channels_ctx: VirtChannelsCtx,
}

impl<ConnectionSeq> Sharee<ConnectionSeq>
where
    ConnectionSeq: ConnectionSM,
{
    pub fn builder(connection_sm: ConnectionSeq) -> ShareeBuilder<ConnectionSeq> {
        ShareeBuilder::new(connection_sm)
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

    pub fn update_without_body<'msg>(&mut self) -> Vec<SMEvent<'msg>> {
        let mut events = SMEvents::new();
        match self.state {
            ShareeState::Connection => {
                self.connection_seq
                    .update_without_message(&mut self.sm_data, &mut events);
                if self.connection_seq.is_terminated() {
                    self.h_go_to_active_state(&mut events);
                }
                self.h_check_for_fatal(&mut events);
            }
            ShareeState::Active => {
                let mut chan_rsps = ChannelResponses::new();
                self.channels_manager
                    .update_without_virt_msg(&mut self.sm_data, &mut events, &mut chan_rsps);
                self.h_map_channels_manager_result(&mut events, chan_rsps);
            }
            ShareeState::Final => {
                events.push(SMEvent::PacketToSend(
                    NowPacket::from_message(NowTerminateMsg::default()).into(),
                ));
            }
        }
        events.unpack()
    }

    pub fn update_with_body<'msg: 'a, 'a>(&mut self, body: &'a NowBody<'msg>) -> Vec<SMEvent<'msg>> {
        let mut events = SMEvents::new();
        match body {
            NowBody::Message(msg) => match self.state {
                ShareeState::Connection => {
                    self.connection_seq
                        .update_with_message(&mut self.sm_data, &mut events, msg);
                    if self.connection_seq.is_terminated() {
                        self.h_go_to_active_state(&mut events);
                    }
                    self.h_check_for_fatal(&mut events);
                }
                ShareeState::Active => {
                    if let NowMessage::Terminate(_) = msg {
                        self.h_transition_state(&mut events, ShareeState::Final);
                    }
                }
                ShareeState::Final => events.push(SMEvent::error(
                    ProtoErrorKind::Sharee(self.state),
                    "unexpected call to `Sharee::update_with_body` in final state with a now message",
                )),
            },
            NowBody::VirtualChannel(chan_msg) => match self.state {
                ShareeState::Connection => events.push(SMEvent::error(
                    ProtoErrorKind::Sharee(self.state),
                    "unexpected call to `Sharee::update_with_body` in connection state with a virtual channel message",
                )),
                ShareeState::Active => {
                    let mut chan_rsps = ChannelResponses::new();
                    self.channels_manager.update_with_virt_msg(
                        &mut self.sm_data,
                        &mut events,
                        &mut chan_rsps,
                        chan_msg,
                    );
                    self.h_map_channels_manager_result(&mut events, chan_rsps);
                }
                ShareeState::Final => events.push(SMEvent::error(
                    ProtoErrorKind::Sharee(self.state),
                    "unexpected call to `Sharee::update_with_body` in final state with a virtual channel message",
                )),
            },
        }
        events.unpack()
    }

    pub fn get_channels_ctx(&self) -> &VirtChannelsCtx {
        &self.channels_ctx
    }

    fn h_check_for_fatal(&mut self, events: &mut SMEvents<'_>) {
        if events.peek().iter().any(|e| matches!(e, SMEvent::Fatal(_))) {
            log::trace!("A fatal error occurred. Set sharee state to final state.");
            self.h_transition_state(events, ShareeState::Final);
        }
    }

    fn h_go_to_active_state(&mut self, events: &mut SMEvents<'_>) {
        log::trace!("enter active state.");
        self.h_transition_state(events, ShareeState::Active);
        for def in &self.sm_data.channel_defs {
            self.channels_ctx.insert(def.flags.value as u8, def.name.clone());
        }
        log::debug!("virtual channels context: {:#?}", self.channels_ctx);
    }

    fn h_map_channels_manager_result<'msg>(&self, events: &mut SMEvents<'msg>, to_send: ChannelResponses<'msg>) {
        for (name, virt_rsp) in to_send.unpack() {
            match self.channels_ctx.get_id_by_channel(&name) {
                Some(channel_id) => events.push(SMEvent::PacketToSend(NowPacket::from_virt_channel(
                    virt_rsp, channel_id,
                ))),
                None => events.push(SMEvent::warn(
                    ProtoErrorKind::ChannelsManager,
                    format!("channel id for {:?} not found in channels context", name),
                )),
            }
        }
    }

    fn h_transition_state(&mut self, events: &mut SMEvents<'_>, state: ShareeState) {
        self.state = state;
        events.push(SMEvent::transition(state));
    }
}

// builder

pub struct ShareeBuilder<ConnectionSeq>
where
    ConnectionSeq: ConnectionSM,
{
    connection_sm: ConnectionSeq,
    supported_auths: Vec<AuthType>,
    capabilities: Vec<NowCapset<'static>>,
    channels_to_open: Vec<NowChannelDef>,
    channels_manager: ChannelsManager,
}

impl<ConnectionSeq> ShareeBuilder<ConnectionSeq>
where
    ConnectionSeq: ConnectionSM,
{
    pub fn new(connection_sm: ConnectionSeq) -> Self {
        Self {
            connection_sm,
            supported_auths: Vec::new(),
            capabilities: Vec::new(),
            channels_to_open: Vec::new(),
            channels_manager: ChannelsManager::default(),
        }
    }

    pub fn supported_auths(self, supported_auths: Vec<AuthType>) -> Self {
        Self {
            supported_auths,
            ..self
        }
    }

    pub fn capabilities(self, capabilities: Vec<NowCapset<'static>>) -> Self {
        Self { capabilities, ..self }
    }

    pub fn channels_to_open(self, channels_to_open: Vec<ChannelName>) -> Self {
        Self {
            channels_to_open: channels_to_open.into_iter().map(NowChannelDef::new).collect(),
            ..self
        }
    }

    pub fn channels_manager(self, channels_manager: ChannelsManager) -> Self {
        Self {
            channels_manager,
            ..self
        }
    }

    pub fn build(self) -> Sharee<ConnectionSeq> {
        Sharee {
            state: ShareeState::Connection,
            connection_seq: self.connection_sm,
            channels_manager: self.channels_manager,
            sm_data: SMData::new(self.supported_auths, self.capabilities, self.channels_to_open),
            channels_ctx: VirtChannelsCtx::new(),
        }
    }
}
