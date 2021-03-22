mod sub_sm;

use crate::error::ProtoErrorKind;
use crate::message::{AuthType, NowChannelDef, NowMessage};
use crate::sm::{ConnectionSM, DummyConnectionSM, ProtoData, ProtoState, SMData, SMEvent, SMEvents};
use alloc::boxed::Box;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct AvailableAuthTypes(Vec<AuthType>);

impl ProtoData for AvailableAuthTypes {}

#[derive(Debug, Clone)]
pub struct Channels(Vec<NowChannelDef>);

impl ProtoData for Channels {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionState {
    Handshake,
    Negotiate,
    Authenticate,
    Associate,
    Capabilities,
    Channels,
    Final,
}

impl ProtoState for ConnectionState {}

pub struct ClientConnectionSeqSM {
    state: ConnectionState,
    current_sm: Box<dyn ConnectionSM>,
    authenticate_sm: Box<dyn ConnectionSM>,
}

impl ClientConnectionSeqSM {
    pub fn new<P: ConnectionSM + 'static>(sm: P) -> Self {
        Self {
            state: ConnectionState::Handshake,
            current_sm: Box::new(sub_sm::HandshakeSM::new()),
            authenticate_sm: Box::new(sm),
        }
    }

    pub fn get_state(&self) -> ConnectionState {
        self.state
    }

    fn __check_for_fatal(&mut self, events: &SMEvents<'_>) {
        if events.peek().iter().any(|e| matches!(e, SMEvent::Fatal(_))) {
            log::trace!("Fatal error occurred. Set connection state machine to final state.");
            self.state = ConnectionState::Final;
        }
    }

    fn __go_to_next_state<'msg>(&mut self, events: &mut SMEvents<'msg>) {
        match self.state {
            ConnectionState::Handshake => {
                self.current_sm = Box::new(sub_sm::NegotiateSM::new());
                self.state = ConnectionState::Negotiate;
                events.push(SMEvent::transition(self.state));
            }
            ConnectionState::Negotiate => {
                core::mem::swap(&mut self.current_sm, &mut self.authenticate_sm);

                // set invalid authenticate_sm field to dummy connection state machine
                let mut dummy_sm: Box<dyn ConnectionSM> = Box::new(DummyConnectionSM);
                core::mem::swap(&mut self.authenticate_sm, &mut dummy_sm);

                self.state = ConnectionState::Authenticate;
                events.push(SMEvent::transition(self.state));
            }
            ConnectionState::Authenticate => {
                self.current_sm = Box::new(sub_sm::AssociateSM::new());
                self.state = ConnectionState::Associate;
                events.push(SMEvent::transition(self.state));
            }
            ConnectionState::Associate => {
                self.current_sm = Box::new(sub_sm::CapabilitiesSM::new());
                self.state = ConnectionState::Capabilities;
                events.push(SMEvent::transition(self.state));
            }
            ConnectionState::Capabilities => {
                self.current_sm = Box::new(sub_sm::ChannelsSM::new());
                self.state = ConnectionState::Channels;
                events.push(SMEvent::transition(self.state));
            }
            ConnectionState::Channels => {
                self.state = ConnectionState::Final;
                events.push(SMEvent::transition(self.state));
            }
            ConnectionState::Final => {
                events.push(SMEvent::warn(
                    ProtoErrorKind::ConnectionSequence(ConnectionState::Final),
                    "Attempted to go to the next state from the final state.",
                ));
            }
        }
    }
}

impl ConnectionSM for ClientConnectionSeqSM {
    fn is_terminated(&self) -> bool {
        self.state == ConnectionState::Final
    }

    fn waiting_for_packet(&self) -> bool {
        self.current_sm.waiting_for_packet()
    }

    fn update_without_message<'msg>(&mut self, data: &mut SMData, events: &mut SMEvents<'msg>) {
        self.current_sm.update_without_message(data, events);
        if self.current_sm.is_terminated() {
            self.__go_to_next_state(events);
        } else {
            self.__check_for_fatal(events);
        }
    }

    fn update_with_message<'msg: 'a, 'a>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        msg: &'a NowMessage<'msg>,
    ) {
        self.current_sm.update_with_message(data, events, msg);
        if self.current_sm.is_terminated() {
            self.__go_to_next_state(events);
        } else {
            self.__check_for_fatal(events);
        }
    }
}
