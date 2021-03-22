use crate::error::{ProtoError, ProtoErrorKind};
use crate::message::{ChannelName, NowVirtualChannel};
use crate::sm::{ChannelResponses, SMData, SMEvent, SMEvents, VirtualChannelSM};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

pub type ChannelsManagerResult<'a> = Result<Option<(ChannelName, NowVirtualChannel<'a>)>, ProtoError>;

pub struct ChannelsManager {
    state_machines: BTreeMap<ChannelName, Box<dyn VirtualChannelSM>>,
}

impl Default for ChannelsManager {
    fn default() -> Self {
        Self {
            state_machines: BTreeMap::new(),
        }
    }
}

impl ChannelsManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sm<VirtChanSM>(mut self, state_machine: VirtChanSM) -> Self
    where
        VirtChanSM: VirtualChannelSM + 'static,
    {
        self.add_sm(state_machine);
        self
    }

    pub fn add_sm<VirtChanSM>(&mut self, state_machine: VirtChanSM) -> Option<Box<dyn VirtualChannelSM>>
    where
        VirtChanSM: VirtualChannelSM + 'static,
    {
        self.state_machines
            .insert(state_machine.get_channel_name(), Box::new(state_machine))
    }

    pub fn update_with_virt_msg<'msg: 'a, 'a>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        to_send: &mut ChannelResponses<'msg>,
        chan_msg: &'a NowVirtualChannel<'msg>,
    ) {
        if let Some(sm) = self.state_machines.get_mut(chan_msg.get_name()) {
            to_send.set_current_channel_name(sm.get_channel_name());
            sm.update_with_chan_msg(data, events, to_send, chan_msg);
        } else {
            events.push(SMEvent::warn(
                ProtoErrorKind::ChannelsManager,
                format!("state machine for channel {:?} not found", chan_msg.get_name()),
            ));
        }
    }

    pub fn update_without_virt_msg<'msg>(
        &mut self,
        data: &mut SMData,
        events: &mut SMEvents<'msg>,
        to_send: &mut ChannelResponses<'msg>,
    ) {
        for sm in self.state_machines.values_mut() {
            if !sm.waiting_for_packet() {
                to_send.set_current_channel_name(sm.get_channel_name());
                sm.update_without_chan_msg(data, events, to_send);
                return;
            }
        }

        events.push(SMEvent::warn(
            ProtoErrorKind::ChannelsManager,
            "no channel state machine is ready to update without message",
        ));
    }

    pub fn waiting_for_packet(&self) -> bool {
        for sm in self.state_machines.values() {
            if !sm.waiting_for_packet() {
                return false;
            }
        }
        true
    }
}
