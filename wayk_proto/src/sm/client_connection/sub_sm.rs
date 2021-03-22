use crate::alloc::string::ToString;
use crate::error::ProtoErrorKind;
use crate::message::{NowActivateMsg, NowCapabilitiesMsg, NowMessage};
use crate::sm::client_connection::{AvailableAuthTypes, Channels};
use crate::sm::{ConnectionSM, ConnectionState, ProtoState, SMData, SMEvent, SMEvents};
use alloc::vec::Vec;
use log::info;

macro_rules! unexpected_call {
    ($sm_struct:ident, $self:ident, $method_name:literal) => {
        SMEvent::fatal(
            ProtoErrorKind::ConnectionSequence($sm_struct::CONNECTION_STATE),
            format!(
                concat!("unexpected call to `{}::", $method_name, "` in state {:?}"),
                $sm_struct::NAME,
                $self.state
            ),
        )
    };
}

macro_rules! unexpected_msg {
    ($sm_struct:ident, $self:ident, $unexpected_msg:ident) => {
        SMEvent::warn(
            ProtoErrorKind::UnexpectedMessage($unexpected_msg.get_type()),
            format!(
                "`{}` received an unexpected message in state {:?}: {:?}",
                $sm_struct::NAME,
                $self.state,
                $unexpected_msg
            ),
        )
    };
}

macro_rules! state_transition {
    ($self:ident, $events:ident, $state:expr) => {
        $self.state = $state;
        $events.push(SMEvent::transition($self.state));
    };
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum BasicState {
    Initial,
    Ready,
    Terminated,
}

impl ProtoState for BasicState {}

// handshake

pub struct HandshakeSM {
    state: BasicState,
}

impl HandshakeSM {
    const CONNECTION_STATE: ConnectionState = ConnectionState::Handshake;
    const NAME: &'static str = "HandshakeSM";

    pub fn new() -> Self {
        Self {
            state: BasicState::Initial,
        }
    }
}

impl ConnectionSM for HandshakeSM {
    fn is_terminated(&self) -> bool {
        self.state == BasicState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == BasicState::Ready
    }

    fn update_without_message<'msg>(&mut self, _: &mut SMData, events: &mut SMEvents<'msg>) {
        use wayk_proto::message::NowHandshakeMsg;

        match self.state {
            BasicState::Initial => {
                events.push(SMEvent::PacketToSend(NowHandshakeMsg::new_success().into()));
                state_transition!(self, events, BasicState::Ready);
            }
            _ => events.push(unexpected_call!(Self, self, "update_without_message")),
        }
    }

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        _: &mut SMData,
        events: &mut SMEvents<'msg>,
        msg: &'a NowMessage<'msg>,
    ) {
        use wayk_proto::message::status::HandshakeStatusCode;

        match self.state {
            BasicState::Ready => match msg {
                NowMessage::Handshake(msg) => match msg.status.code() {
                    HandshakeStatusCode::Success => {
                        log::trace!("handshake succeeded");
                        state_transition!(self, events, BasicState::Terminated);
                    }
                    HandshakeStatusCode::Failure => events.push(SMEvent::fatal(
                        ProtoErrorKind::ConnectionSequence(ConnectionState::Handshake),
                        "handshake failed",
                    )),
                    HandshakeStatusCode::Incompatible => events.push(SMEvent::fatal(
                        ProtoErrorKind::ConnectionSequence(ConnectionState::Handshake),
                        "version incompatible",
                    )),
                    HandshakeStatusCode::Other(code) => events.push(SMEvent::error(
                        ProtoErrorKind::ConnectionSequence(ConnectionState::Handshake),
                        format!("handshake status code: {}", code),
                    )),
                },
                unexpected => events.push(unexpected_msg!(Self, self, unexpected)),
            },
            _ => events.push(unexpected_call!(Self, self, "update_with_message")),
        }
    }
}

// negotiate

pub struct NegotiateSM {
    state: BasicState,
}

impl NegotiateSM {
    const CONNECTION_STATE: ConnectionState = ConnectionState::Negotiate;
    const NAME: &'static str = "NegotiateSM";

    pub fn new() -> Self {
        Self {
            state: BasicState::Initial,
        }
    }
}

impl ConnectionSM for NegotiateSM {
    fn is_terminated(&self) -> bool {
        self.state == BasicState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == BasicState::Ready
    }

    fn update_without_message<'msg>(&mut self, data: &mut SMData, events: &mut SMEvents<'msg>) {
        use wayk_proto::message::{NegotiateFlags, NowNegotiateMsg};

        match &self.state {
            BasicState::Initial => {
                events.push(SMEvent::PacketToSend(
                    NowNegotiateMsg::new_with_auth_list(
                        NegotiateFlags::new_empty().set_srp_extended(),
                        data.supported_auths.clone(),
                    )
                    .into(),
                ));
                state_transition!(self, events, BasicState::Ready);
            }
            _ => events.push(unexpected_call!(Self, self, "update_without_message")),
        }
    }

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        msg: &'a NowMessage<'msg>,
    ) {
        match &self.state {
            BasicState::Ready => match msg {
                NowMessage::Negotiate(msg) => {
                    info!("Available authentication methods on server: {:?}", msg.auth_list.0);

                    let common_auth_types = msg
                        .auth_list
                        .iter()
                        .filter(|elem| data.supported_auths.contains(elem))
                        .copied()
                        .collect();

                    events.push(SMEvent::data(AvailableAuthTypes(common_auth_types)));

                    state_transition!(self, events, BasicState::Terminated);
                }
                unexpected => events.push(unexpected_msg!(Self, self, unexpected)),
            },
            _ => events.push(unexpected_call!(Self, self, "update_with_message")),
        }
    }
}

// associate

#[derive(PartialEq, Debug, Clone, Copy)]
enum AssociateState {
    WaitInfo,
    WaitResponse,
    Terminated,
}

impl ProtoState for AssociateState {}

pub struct AssociateSM {
    state: AssociateState,
}

impl AssociateSM {
    const CONNECTION_STATE: ConnectionState = ConnectionState::Associate;
    const NAME: &'static str = "AssociateSM";

    pub fn new() -> Self {
        Self {
            state: AssociateState::WaitInfo,
        }
    }
}

impl ConnectionSM for AssociateSM {
    fn is_terminated(&self) -> bool {
        self.state == AssociateState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        !self.is_terminated()
    }

    fn update_without_message<'msg>(&mut self, _: &mut SMData, events: &mut SMEvents<'msg>) {
        events.push(unexpected_call!(Self, self, "update_without_message"));
    }

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        _: &mut SMData,
        events: &mut SMEvents<'msg>,
        msg: &'a NowMessage<'msg>,
    ) {
        use wayk_proto::message::status::AssociateStatusCode;
        use wayk_proto::message::NowAssociateMsg;

        match &self.state {
            AssociateState::WaitInfo => match msg {
                NowMessage::Associate(NowAssociateMsg::Info(msg)) => {
                    if msg.flags.active() {
                        log::trace!("associate process session is already active");
                    } else {
                        events.push(SMEvent::PacketToSend(NowAssociateMsg::new_request().into()));
                    }
                    state_transition!(self, events, AssociateState::WaitResponse);
                }
                unexpected => events.push(unexpected_msg!(Self, self, unexpected)),
            },
            AssociateState::WaitResponse => match msg {
                NowMessage::Associate(NowAssociateMsg::Response(msg)) => match msg.status.code() {
                    AssociateStatusCode::Success => {
                        state_transition!(self, events, AssociateState::Terminated);
                        log::trace!("associate process succeeded");
                    }
                    AssociateStatusCode::Failure => events.push(SMEvent::fatal(
                        ProtoErrorKind::ConnectionSequence(ConnectionState::Associate),
                        format!("Association failed {:?}", msg.status.status_type().to_string()),
                    )),
                    AssociateStatusCode::Other(code) => events.push(SMEvent::error(
                        ProtoErrorKind::ConnectionSequence(ConnectionState::Associate),
                        format!("Associate status code: {}", code),
                    )),
                },
                unexpected => events.push(unexpected_msg!(Self, self, unexpected)),
            },
            AssociateState::Terminated => events.push(unexpected_call!(Self, self, "update_with_message")),
        }
    }
}

// capabilities

pub struct CapabilitiesSM {
    state: BasicState,
}

impl CapabilitiesSM {
    const CONNECTION_STATE: ConnectionState = ConnectionState::Capabilities;
    const NAME: &'static str = "CapabilitiesSM";

    pub fn new() -> Self {
        Self {
            state: BasicState::Ready,
        }
    }
}

impl ConnectionSM for CapabilitiesSM {
    fn is_terminated(&self) -> bool {
        self.state == BasicState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state != BasicState::Terminated
    }

    fn update_without_message<'msg>(&mut self, _: &mut SMData, events: &mut SMEvents<'msg>) {
        events.push(unexpected_call!(Self, self, "update_without_message"));
    }

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        msg: &'a NowMessage<'msg>,
    ) {
        if self.state == BasicState::Terminated {
            events.push(unexpected_call!(Self, self, "update_with_message"));
        } else {
            match msg {
                NowMessage::Capabilities(msg) => {
                    log::info!(
                        "Server capabilities (short): {:?}",
                        msg.capabilities
                            .iter()
                            .map(|caps| caps.name_as_str())
                            .collect::<Vec<&str>>()
                    );
                    log::trace!("Server capabilities details: {:#?}", msg.capabilities.0);

                    events.push(SMEvent::PacketToSend(
                        NowCapabilitiesMsg::new_with_capabilities(data.capabilities.clone()).into(),
                    ));
                    state_transition!(self, events, BasicState::Terminated);
                }
                unexpected => events.push(unexpected_msg!(Self, self, unexpected)),
            }
        }
    }
}

// channels

#[derive(PartialEq, Debug, Clone, Copy)]
enum ChannelPairingState {
    SendListRequest,
    WaitListResponse,
    SendOpenRequest,
    WaitOpenResponse,
    Terminated,
}

impl ProtoState for ChannelPairingState {}

pub struct ChannelsSM {
    state: ChannelPairingState,
}

impl ChannelsSM {
    const CONNECTION_STATE: ConnectionState = ConnectionState::Channels;
    const NAME: &'static str = "ChannelsSM";

    pub fn new() -> Self {
        Self {
            state: ChannelPairingState::SendListRequest,
        }
    }
}

impl ConnectionSM for ChannelsSM {
    fn is_terminated(&self) -> bool {
        self.state == ChannelPairingState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == ChannelPairingState::WaitListResponse || self.state == ChannelPairingState::WaitOpenResponse
    }

    fn update_without_message<'msg>(&mut self, data: &mut SMData, events: &mut SMEvents<'msg>) {
        use crate::message::{ChannelMessageType, NowChannelMsg};
        match self.state {
            ChannelPairingState::SendListRequest => {
                events.push(SMEvent::PacketToSend(
                    NowChannelMsg::new(ChannelMessageType::ChannelListRequest, data.channel_defs.clone()).into(),
                ));
                state_transition!(self, events, ChannelPairingState::WaitListResponse);
            }
            ChannelPairingState::WaitListResponse => {
                events.push(unexpected_call!(Self, self, "update_without_message"))
            }
            ChannelPairingState::SendOpenRequest => {
                events.push(SMEvent::PacketToSend(
                    NowChannelMsg::new(ChannelMessageType::ChannelOpenRequest, data.channel_defs.clone()).into(),
                ));
                state_transition!(self, events, ChannelPairingState::WaitOpenResponse);
            }
            ChannelPairingState::WaitOpenResponse => {
                events.push(unexpected_call!(Self, self, "update_without_message"))
            }
            ChannelPairingState::Terminated => events.push(unexpected_call!(Self, self, "update_without_message")),
        }
    }

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        msg: &'a NowMessage<'msg>,
    ) {
        use crate::message::ChannelName;

        match self.state {
            ChannelPairingState::SendListRequest => events.push(unexpected_call!(Self, self, "update_with_message")),
            ChannelPairingState::WaitListResponse => match msg {
                NowMessage::Channel(msg) => {
                    log::info!(
                        "Available channel(s) on server: {:?}",
                        msg.channel_list
                            .iter()
                            .map(|def| &def.name)
                            .collect::<Vec<&ChannelName>>()
                    );

                    let mut unavailable_channels = Vec::new();
                    for def in data.channel_defs.iter() {
                        if !msg.channel_list.iter().any(|d| d.name == def.name) {
                            unavailable_channels.push(def.name.clone())
                        }
                    }

                    if !unavailable_channels.is_empty() {
                        events.push(SMEvent::warn(
                            ProtoErrorKind::ConnectionSequence(Self::CONNECTION_STATE),
                            format!("Unavailable channel(s) on server ignored: {:?}", unavailable_channels),
                        ));
                        data.channel_defs
                            .retain(|def| !unavailable_channels.contains(&def.name));
                    }

                    events.push(SMEvent::data(Channels(data.channel_defs.clone())));
                    state_transition!(self, events, ChannelPairingState::SendOpenRequest);
                }
                unexpected => events.push(unexpected_msg!(Self, self, unexpected)),
            },
            ChannelPairingState::SendOpenRequest => events.push(unexpected_call!(Self, self, "update_with_message")),
            ChannelPairingState::WaitOpenResponse => match msg {
                NowMessage::Channel(msg) => {
                    log::info!(
                        "Opened channel(s): {:?}",
                        msg.channel_list
                            .iter()
                            .map(|def| &def.name)
                            .collect::<Vec<&ChannelName>>()
                    );

                    data.channel_defs = msg.channel_list.0.clone();

                    events.push(SMEvent::PacketToSend(NowActivateMsg::default().into()));
                    state_transition!(self, events, ChannelPairingState::Terminated);
                }
                unexpected => events.push(unexpected_msg!(Self, self, unexpected)),
            },
            ChannelPairingState::Terminated => events.push(unexpected_call!(Self, self, "update_with_message")),
        }
    }
}
