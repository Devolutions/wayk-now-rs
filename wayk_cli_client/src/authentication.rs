use crate::config::AuthConfig;
use wayk_proto::auth::pfp::NowAuthPFP;
use wayk_proto::error::ProtoErrorKind;
use wayk_proto::message::{NowAuthenticateMsg, NowMessage};
use wayk_proto::sm::{ConnectionSM, ConnectionState, ProtoState, SMData, SMEvent, SMEvents};

#[derive(Debug, PartialEq, Clone, Copy)]
enum AuthState {
    Initial,
    Ongoing,
    PostAuth,
    Terminated,
}

impl ProtoState for AuthState {}

pub struct AuthenticateSM {
    state: AuthState,
    auth_config: AuthConfig,
}

impl AuthenticateSM {
    pub fn new(auth_config: AuthConfig) -> Self {
        Self {
            state: AuthState::Initial,
            auth_config,
        }
    }

    fn h_transition_state(&mut self, events: &mut SMEvents<'_>, state: AuthState) {
        self.state = state;
        events.push(SMEvent::transition(state));
    }
}

impl ConnectionSM for AuthenticateSM {
    fn is_terminated(&self) -> bool {
        self.state == AuthState::Terminated
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == AuthState::PostAuth
    }

    fn update_without_message<'msg>(&mut self, data: &mut SMData, events: &mut SMEvents<'msg>) {
        match &self.state {
            AuthState::Initial => {
                if data.supported_auths.contains(&self.auth_config.auth_type()) {
                    self.h_transition_state(events, AuthState::Ongoing);
                } else {
                    events.push(SMEvent::error(
                        ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate),
                        format!(
                            "authentication method `{:?}` not available on server.",
                            self.auth_config.auth_type()
                        ),
                    ))
                }
            }
            AuthState::Ongoing => match &self.auth_config {
                AuthConfig::PFP(conf) => {
                    match NowAuthPFP::new_owned_negotiate_token(&conf.friendly_name, &conf.friendly_text) {
                        Ok(msg) => {
                            events.push(SMEvent::PacketToSend(msg.into()));
                            self.h_transition_state(events, AuthState::PostAuth);
                        }
                        Err(e) => {
                            events.push(SMEvent::Error(e));
                        }
                    }
                }
                AuthConfig::None => {}
            },
            state => events.push(SMEvent::error(
                ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate),
                format!(
                    "unexpected call to `AuthenticateSM::update_without_message` in state {:?}",
                    state
                ),
            )),
        }
    }

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        _: &mut SMData,
        events: &mut SMEvents<'msg>,
        msg: &'a NowMessage<'msg>,
    ) {
        match &self.state {
            AuthState::PostAuth => {
                self.h_transition_state(events, AuthState::Terminated);
                match msg {
                    NowMessage::Authenticate(NowAuthenticateMsg::Success(_)) => {
                        log::trace!("authenticate process succeeded.")
                    }
                    NowMessage::Authenticate(NowAuthenticateMsg::Failure(_)) => events.push(SMEvent::error(
                        ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate),
                        "authenticate process failed",
                    )),
                    unexpected => events.push(SMEvent::warn(
                        ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate),
                        format!("received an unexpected message: {:?}", unexpected),
                    )),
                }
            }
            state => events.push(SMEvent::warn(
                ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate),
                format!(
                    "unexpected call to `AuthenticateSM::update_with_message` in state {:?}",
                    state
                ),
            )),
        }
    }
}
