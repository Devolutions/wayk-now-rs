use super::{ConnectionSM, ConnectionSMResult};
use crate::{
    error::{ProtoError, ProtoErrorKind, ProtoErrorResultExt},
    message::{NowActivateMsg, NowCapabilitiesMsg, NowMessage},
    sm::{ConnectionSMSharedData, ConnectionSMSharedDataRc, ConnectionState},
};
use log::info;
use std::{cell::RefCell, rc::Rc};

macro_rules! unexpected_call {
    ($sm_struct:ident, $self:ident, $method_name:literal) => {
        ProtoError::new(ProtoErrorKind::ConnectionSequence($sm_struct::CONNECTION_STATE)).or_desc(format!(
            concat!("unexpected call to `{}::", $method_name, "` in state {:?}"),
            $sm_struct::NAME,
            $self.state
        ))
    };
}

macro_rules! unexpected_msg {
    ($sm_struct:ident, $self:ident, $unexpected_msg:ident) => {
        ProtoError::new(ProtoErrorKind::UnexpectedMessage($unexpected_msg.get_type())).or_desc(format!(
            "`{}` received an unexpected message in state {:?}: {:?}",
            $sm_struct::NAME,
            $self.state,
            $unexpected_msg
        ))
    };
}

#[derive(PartialEq, Debug)]
enum BasicState {
    Initial,
    Ready,
    Terminated,
}

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
    fn set_shared_data(&mut self, _: ConnectionSMSharedDataRc) {}

    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc> {
        None
    }

    fn is_terminated(&self) -> bool {
        self.state == BasicState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == BasicState::Ready
    }

    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg> {
        use wayk_proto::message::NowHandshakeMsg;

        match &self.state {
            BasicState::Initial => {
                self.state = BasicState::Ready;
                Ok(Some(NowHandshakeMsg::new_success().into()))
            }
            _ => unexpected_call!(Self, self, "update_without_message"),
        }
    }

    fn update_with_message<'msg: 'a, 'a>(&mut self, msg: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg> {
        use wayk_proto::message::status::HandshakeStatusCode;

        match &self.state {
            BasicState::Ready => match msg {
                NowMessage::Handshake(msg) => match msg.status.code() {
                    HandshakeStatusCode::Success => {
                        log::trace!("handshake succeeded");
                        self.state = BasicState::Terminated;
                        Ok(None)
                    }
                    HandshakeStatusCode::Failure => {
                        ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Handshake))
                            .or_desc("handshake failed")
                    }
                    HandshakeStatusCode::Incompatible => {
                        ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Handshake))
                            .or_desc("version incompatible")
                    }
                },
                unexpected => unexpected_msg!(Self, self, unexpected),
            },
            _ => unexpected_call!(Self, self, "update_with_message"),
        }
    }
}

// negotiate

pub struct NegotiateSM {
    state: BasicState,
    shared_data: Rc<RefCell<ConnectionSMSharedData>>,
}

impl NegotiateSM {
    const CONNECTION_STATE: ConnectionState = ConnectionState::Negotiate;
    const NAME: &'static str = "NegotiateSM";

    pub fn new(shared_data: Rc<RefCell<ConnectionSMSharedData>>) -> Self {
        Self {
            state: BasicState::Initial,
            shared_data,
        }
    }
}

impl ConnectionSM for NegotiateSM {
    fn set_shared_data(&mut self, shared_data: Rc<RefCell<ConnectionSMSharedData>>) {
        self.shared_data = shared_data;
    }

    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc> {
        Some(Rc::clone(&self.shared_data))
    }

    fn is_terminated(&self) -> bool {
        self.state == BasicState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == BasicState::Ready
    }

    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg> {
        use wayk_proto::message::{NegotiateFlags, NowNegotiateMsg};

        match &self.state {
            BasicState::Initial => {
                self.state = BasicState::Ready;
                let shared_data = self.shared_data.borrow();
                Ok(Some(
                    NowNegotiateMsg::new_with_auth_list(
                        NegotiateFlags::new_empty().set_srp_extended(),
                        shared_data.available_auth_types.clone(),
                    )
                    .into(),
                ))
            }
            _ => unexpected_call!(Self, self, "update_without_message"),
        }
    }

    fn update_with_message<'msg: 'a, 'a>(&mut self, msg: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg> {
        match &self.state {
            BasicState::Ready => match msg {
                NowMessage::Negotiate(msg) => {
                    info!("Available authentication methods on server: {:?}", msg.auth_list.0);

                    let mut shared_data = self.shared_data.borrow_mut();
                    let common_auth_types = msg
                        .auth_list
                        .iter()
                        .filter(|elem| shared_data.available_auth_types.contains(elem))
                        .copied()
                        .collect();
                    shared_data.available_auth_types = common_auth_types;

                    self.state = BasicState::Terminated;
                    Ok(None)
                }
                unexpected => unexpected_msg!(Self, self, unexpected),
            },
            _ => unexpected_call!(Self, self, "update_with_message"),
        }
    }
}

// associate

#[derive(PartialEq, Debug)]
enum AssociateState {
    WaitInfo,
    WaitResponse,
    Terminated,
}

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
    fn set_shared_data(&mut self, _: ConnectionSMSharedDataRc) {}

    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc> {
        None
    }

    fn is_terminated(&self) -> bool {
        self.state == AssociateState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        !self.is_terminated()
    }

    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg> {
        unexpected_call!(Self, self, "update_without_message")
    }

    fn update_with_message<'msg: 'a, 'a>(&mut self, msg: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg> {
        use wayk_proto::message::{status::AssociateStatusCode, NowAssociateMsg};

        match &self.state {
            AssociateState::WaitInfo => match msg {
                NowMessage::Associate(NowAssociateMsg::Info(msg)) => {
                    self.state = AssociateState::WaitResponse;
                    if msg.flags.active() {
                        log::trace!("associate process session is already active");
                        Ok(None)
                    } else {
                        Ok(Some(NowAssociateMsg::new_request().into()))
                    }
                }
                unexpected => unexpected_msg!(Self, self, unexpected),
            },
            AssociateState::WaitResponse => match msg {
                NowMessage::Associate(NowAssociateMsg::Response(msg)) => match msg.status.code() {
                    AssociateStatusCode::Success => {
                        self.state = AssociateState::Terminated;
                        log::trace!("associate process succeeded");
                        Ok(None)
                    }
                    AssociateStatusCode::Failure => {
                        ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Handshake))
                            .or_desc(format!("Association failed {:?}", msg.status.status_type().to_string()))
                    }
                },
                unexpected => unexpected_msg!(Self, self, unexpected),
            },
            AssociateState::Terminated => unexpected_call!(Self, self, "update_with_message"),
        }
    }
}

// capabilities

pub struct CapabilitiesSM {
    terminated: bool,
    shared_data: ConnectionSMSharedDataRc,
}

impl CapabilitiesSM {
    pub fn new(shared_data: ConnectionSMSharedDataRc) -> Self {
        Self {
            terminated: false,
            shared_data,
        }
    }
}

impl ConnectionSM for CapabilitiesSM {
    fn set_shared_data(&mut self, shared_data: ConnectionSMSharedDataRc) {
        self.shared_data = shared_data;
    }

    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc> {
        Some(Rc::clone(&self.shared_data))
    }

    fn is_terminated(&self) -> bool {
        self.terminated
    }

    fn waiting_for_packet(&self) -> bool {
        !self.terminated
    }

    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg> {
        ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Capabilities))
            .or_desc("unexpected call to `CapabilitiesSM::update_without_message`")
    }

    fn update_with_message<'msg: 'a, 'a>(&mut self, msg: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg> {
        if self.terminated {
            ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Capabilities))
                .or_desc("unexpected call to `CapabilitiesSM::update_with_message` in terminated state")
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

                    self.terminated = true;
                    Ok(Some(
                        NowCapabilitiesMsg::new_with_capabilities(self.shared_data.borrow().capabilities.clone())
                            .into(),
                    ))
                }
                unexpected => ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Capabilities))
                    .or_desc(format!("received an unexpected message: {:?}", unexpected)),
            }
        }
    }
}

// channels

#[derive(PartialEq, Debug)]
enum ChannelPairingState {
    SendListRequest,
    WaitListResponse,
    SendOpenRequest,
    WaitOpenResponse,
    Terminated,
}

pub struct ChannelsSM {
    state: ChannelPairingState,
    shared_data: ConnectionSMSharedDataRc,
}

impl ChannelsSM {
    const CONNECTION_STATE: ConnectionState = ConnectionState::Channels;
    const NAME: &'static str = "ChannelsSM";

    pub fn new(shared_data: ConnectionSMSharedDataRc) -> Self {
        Self {
            state: ChannelPairingState::SendListRequest,
            shared_data,
        }
    }
}

impl ConnectionSM for ChannelsSM {
    fn set_shared_data(&mut self, shared_data: ConnectionSMSharedDataRc) {
        self.shared_data = shared_data;
    }

    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc> {
        Some(Rc::clone(&self.shared_data))
    }

    fn is_terminated(&self) -> bool {
        self.state == ChannelPairingState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == ChannelPairingState::WaitListResponse || self.state == ChannelPairingState::WaitOpenResponse
    }

    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg> {
        use crate::message::{ChannelMessageType, NowChannelMsg};
        match self.state {
            ChannelPairingState::SendListRequest => {
                self.state = ChannelPairingState::WaitListResponse;
                Ok(Some(
                    NowChannelMsg::new(
                        ChannelMessageType::ChannelListRequest,
                        self.shared_data.borrow().channels.clone(),
                    )
                    .into(),
                ))
            }
            ChannelPairingState::WaitListResponse => unexpected_call!(Self, self, "update_without_message"),
            ChannelPairingState::SendOpenRequest => {
                self.state = ChannelPairingState::WaitOpenResponse;
                Ok(Some(
                    NowChannelMsg::new(
                        ChannelMessageType::ChannelOpenRequest,
                        self.shared_data.borrow().channels.clone(),
                    )
                    .into(),
                ))
            }
            ChannelPairingState::WaitOpenResponse => unexpected_call!(Self, self, "update_without_message"),
            ChannelPairingState::Terminated => unexpected_call!(Self, self, "update_without_message"),
        }
    }

    fn update_with_message<'msg: 'a, 'a>(&mut self, msg: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg> {
        use crate::message::ChannelName;

        match self.state {
            ChannelPairingState::SendListRequest => unexpected_call!(Self, self, "update_with_message"),
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
                    for def in self.shared_data.borrow().channels.iter() {
                        if !msg.channel_list.iter().any(|d| d.name == def.name) {
                            unavailable_channels.push(def.name.clone())
                        }
                    }

                    if !unavailable_channels.is_empty() {
                        log::warn!("Unavailable channel(s) on server ignored: {:?}", unavailable_channels);
                        self.shared_data
                            .borrow_mut()
                            .channels
                            .retain(|def| !unavailable_channels.contains(&def.name));
                    }

                    self.state = ChannelPairingState::SendOpenRequest;
                    Ok(None)
                }
                unexpected => unexpected_msg!(Self, self, unexpected),
            },
            ChannelPairingState::SendOpenRequest => unexpected_call!(Self, self, "update_with_message"),
            ChannelPairingState::WaitOpenResponse => match msg {
                NowMessage::Channel(msg) => {
                    log::info!(
                        "Opened channel(s): {:?}",
                        msg.channel_list
                            .iter()
                            .map(|def| &def.name)
                            .collect::<Vec<&ChannelName>>()
                    );
                    self.state = ChannelPairingState::Terminated;
                    self.shared_data.borrow_mut().channels = msg.channel_list.0.clone();
                    Ok(Some(NowActivateMsg::default().into()))
                }
                unexpected => unexpected_msg!(Self, self, unexpected),
            },
            ChannelPairingState::Terminated => unexpected_call!(Self, self, "update_with_message"),
        }
    }
}
