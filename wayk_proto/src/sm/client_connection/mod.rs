/** client connection sequence **/
mod sub_sm;

use crate::{
    message::{AuthType, ChannelName, NowCapset, NowChannelDef, NowMessage},
    sm::{
        ConnectionSM, ConnectionSMResult, ConnectionSMSharedData, ConnectionSMSharedDataRc, ConnectionSeqCallbackTrait,
        ConnectionState, DummyConnectionSM,
    },
};
use std::{cell::RefCell, rc::Rc};

pub struct ClientConnectionSeqSM<UserCallback> {
    user_callback: UserCallback,
    state: ConnectionState,
    current_sm: Box<dyn ConnectionSM>,
    authenticate_sm: Box<dyn ConnectionSM>,
    shared_data: ConnectionSMSharedDataRc,
}

impl<UserCallback> ClientConnectionSeqSM<UserCallback>
where
    UserCallback: ConnectionSeqCallbackTrait,
{
    pub fn builder(user_callback: UserCallback) -> ClientConnectionSeqBuilder<UserCallback> {
        ClientConnectionSeqBuilder {
            user_callback,
            available_auth_types: Vec::new(),
            authenticate_sm: Box::new(DummyConnectionSM),
            capabilities: Vec::new(),
            channels_to_open: Vec::new(),
        }
    }

    pub fn new(
        user_callback: UserCallback,
        available_auth_types: Vec<AuthType>,
        authenticate_sm: Box<dyn ConnectionSM>,
        capabilities: Vec<NowCapset<'static>>,
        channels_to_open: Vec<NowChannelDef>,
    ) -> Self {
        Self {
            user_callback,
            state: ConnectionState::Handshake,
            current_sm: Box::new(sub_sm::HandshakeSM::new()),
            authenticate_sm,
            shared_data: Rc::new(RefCell::new(ConnectionSMSharedData {
                available_auth_types,
                capabilities,
                channels: channels_to_open,
            })),
        }
    }

    pub fn get_state(&self) -> ConnectionState {
        self.state
    }

    fn __check_result(&mut self, result: &ConnectionSMResult<'_>) {
        if result.is_err() {
            log::trace!("an error occurred. Set connection state machine to final state.");
            self.state = ConnectionState::Final;
        }
    }

    fn __go_to_next_state(&mut self) {
        match self.state {
            ConnectionState::Handshake => {
                self.state = ConnectionState::Negotiate;
                self.current_sm = Box::new(sub_sm::NegotiateSM::new(Rc::clone(&self.shared_data)));
                self.user_callback.on_handshake_completed(&self.shared_data.borrow());
            }
            ConnectionState::Negotiate => {
                self.state = ConnectionState::Authenticate;
                self.authenticate_sm.set_shared_data(Rc::clone(&self.shared_data));
                std::mem::swap(&mut self.current_sm, &mut self.authenticate_sm);

                // set invalid authenticate_sm field to dummy connection state machine
                let mut dummy_sm: Box<dyn ConnectionSM> = Box::new(DummyConnectionSM);
                std::mem::swap(&mut self.authenticate_sm, &mut dummy_sm);

                self.user_callback.on_negotiate_completed(&self.shared_data.borrow());
            }
            ConnectionState::Authenticate => {
                self.state = ConnectionState::Associate;
                self.current_sm = Box::new(sub_sm::AssociateSM::new());
                self.user_callback.on_authenticate_completed(&self.shared_data.borrow());
            }
            ConnectionState::Associate => {
                self.state = ConnectionState::Capabilities;
                self.current_sm = Box::new(sub_sm::CapabilitiesSM::new(Rc::clone(&self.shared_data)));
                self.user_callback.on_associate_completed(&self.shared_data.borrow());
            }
            ConnectionState::Capabilities => {
                self.state = ConnectionState::Channels;
                self.current_sm = Box::new(sub_sm::ChannelsSM::new(Rc::clone(&self.shared_data)));
                self.user_callback.on_capabilities_completed(&self.shared_data.borrow());
            }
            ConnectionState::Channels => {
                self.state = ConnectionState::Final;
                self.user_callback.on_connection_completed(&self.shared_data.borrow());
            }
            ConnectionState::Final => log::warn!("Attempted to go to the next state from the final state."),
        }
    }
}

impl<UserCallback> ConnectionSM for ClientConnectionSeqSM<UserCallback>
where
    UserCallback: ConnectionSeqCallbackTrait,
{
    fn set_shared_data(&mut self, shared_data: ConnectionSMSharedDataRc) {
        self.shared_data = shared_data
    }

    fn get_shared_data(&self) -> Option<ConnectionSMSharedDataRc> {
        Some(Rc::clone(&self.shared_data))
    }

    fn is_terminated(&self) -> bool {
        self.state == ConnectionState::Final
    }

    fn waiting_for_packet(&self) -> bool {
        self.current_sm.waiting_for_packet()
    }

    fn update_without_message<'msg>(&mut self) -> ConnectionSMResult<'msg> {
        let response = self.current_sm.update_without_message();

        if self.current_sm.is_terminated() {
            self.__go_to_next_state();
        } else {
            self.__check_result(&response);
        }

        response
    }

    fn update_with_message<'msg: 'a, 'a>(&mut self, msg: &'a NowMessage<'msg>) -> ConnectionSMResult<'msg> {
        let response = self.current_sm.update_with_message(msg);

        if self.current_sm.is_terminated() {
            self.__go_to_next_state();
        } else {
            self.__check_result(&response);
        }

        response
    }
}

// builder

pub struct ClientConnectionSeqBuilder<UserCallback> {
    available_auth_types: Vec<AuthType>,
    authenticate_sm: Box<dyn ConnectionSM>,
    capabilities: Vec<NowCapset<'static>>,
    channels_to_open: Vec<NowChannelDef>,
    user_callback: UserCallback,
}

impl<UserCallback> ClientConnectionSeqBuilder<UserCallback>
where
    UserCallback: ConnectionSeqCallbackTrait,
{
    pub fn available_auth_process(self, available_auth_types: Vec<AuthType>) -> Self {
        Self {
            available_auth_types,
            ..self
        }
    }

    pub fn authenticate_sm<P: ConnectionSM + 'static>(self, sm: P) -> Self {
        Self {
            authenticate_sm: Box::new(sm),
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

    pub fn build(self) -> ClientConnectionSeqSM<UserCallback> {
        ClientConnectionSeqSM::new(
            self.user_callback,
            self.available_auth_types,
            self.authenticate_sm,
            self.capabilities,
            self.channels_to_open,
        )
    }
}
