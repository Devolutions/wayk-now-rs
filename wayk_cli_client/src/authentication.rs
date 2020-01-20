use crate::config::AuthConfig;
use std::rc::Rc;
use wayk_proto::{
    auth::pfp::NowAuthPFP,
    error::{ProtoError, ProtoErrorKind, ProtoErrorResultExt},
    message::{NowAuthenticateMsg, NowMessage},
    sm::{ConnectionSM, ConnectionSMResult, ConnectionSMSharedDataRc, ConnectionState},
};

#[derive(Debug, PartialEq)]
enum AuthState {
    Initial,
    Ongoing,
    PostAuth,
    Terminated,
}

pub struct AuthenticateSM {
    state: AuthState,
    shared_data: Option<ConnectionSMSharedDataRc>,
    auth_config: AuthConfig,
}

impl AuthenticateSM {
    pub fn new(auth_config: AuthConfig) -> Self {
        Self {
            state: AuthState::Initial,
            shared_data: None,
            auth_config,
        }
    }
}

impl ConnectionSM for AuthenticateSM {
    fn is_terminated(&self) -> bool {
        self.state == AuthState::Terminated
    }

    fn set_shared_data(&mut self, shared_data: ConnectionSMSharedDataRc) {
        self.shared_data = Some(shared_data);
    }

    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc> {
        self.shared_data.as_ref().map(Rc::clone)
    }

    fn waiting_for_packet(&self) -> bool {
        self.state == AuthState::PostAuth
    }

    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg> {
        let shared_data = if let Some(shared_data) = &self.shared_data {
            shared_data
        } else {
            return ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate))
                .or_desc("AuthenticateSM: shared data are missing");
        };

        match &self.state {
            AuthState::Initial => {
                if shared_data
                    .borrow()
                    .available_auth_types
                    .contains(&self.auth_config.auth_type())
                {
                    self.state = AuthState::Ongoing;
                    Ok(None)
                } else {
                    ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate)).or_desc(format!(
                        "authentication method `{:?}` not available on server.",
                        self.auth_config.auth_type()
                    ))
                }
            }
            AuthState::Ongoing => {
                self.state = AuthState::PostAuth;
                match &self.auth_config {
                    AuthConfig::PFP(conf) => Ok(Some(
                        NowAuthPFP::new_owned_negotiate_token(&conf.friendly_name, &conf.friendly_text)?.into(),
                    )),
                    AuthConfig::None => Ok(None),
                }
            }
            state => {
                ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate)).or_desc(format!(
                    "unexpected call to `AuthenticateSM::update_without_message` in state {:?}",
                    state
                ))
            }
        }
    }

    fn update_with_message<'msg: 'a, 'a>(&mut self, msg: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg> {
        match &self.state {
            AuthState::PostAuth => {
                self.state = AuthState::Terminated;
                match msg {
                    NowMessage::Authenticate(NowAuthenticateMsg::Success(_)) => {
                        log::trace!("authenticate process succeeded.");
                        Ok(None)
                    }
                    NowMessage::Authenticate(NowAuthenticateMsg::Failure(_)) => {
                        ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate))
                            .or_desc("authenticate process failed.")
                    }
                    unexpected => ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate))
                        .or_desc(format!("received an unexpected message: {:?}", unexpected)),
                }
            }

            state => {
                ProtoError::new(ProtoErrorKind::ConnectionSequence(ConnectionState::Authenticate)).or_desc(format!(
                    "unexpected call to `AuthenticateSM::update_with_message` in state {:?}",
                    state
                ))
            }
        }
    }
}
