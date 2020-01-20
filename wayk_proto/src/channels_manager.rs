use crate::{
    error::{ProtoError, ProtoErrorKind, ProtoErrorResultExt},
    message::{ChannelName, NowVirtualChannel},
    sm::VirtualChannelSM,
};
use alloc::collections::BTreeMap;

pub type ChannelsManagerResult<'a> = Result<Option<(ChannelName, NowVirtualChannel<'a>)>, ProtoError>;

pub struct ChannelsManager {
    state_machines: BTreeMap<ChannelName, Box<dyn VirtualChannelSM>>,
}

impl Default for ChannelsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelsManager {
    pub fn new() -> Self {
        Self {
            state_machines: BTreeMap::new(),
        }
    }

    pub fn with_sm<VirtChanSM>(mut self, state_machine: VirtChanSM) -> Self
    where
        VirtChanSM: VirtualChannelSM + 'static,
    {
        self.add_channel_sm(state_machine);
        self
    }

    pub fn add_channel_sm<VirtChanSM>(&mut self, state_machine: VirtChanSM) -> Option<Box<dyn VirtualChannelSM>>
    where
        VirtChanSM: VirtualChannelSM + 'static,
    {
        self.state_machines
            .insert(state_machine.get_channel_name(), Box::new(state_machine))
    }

    pub fn update_with_virt_msg<'msg: 'a, 'a>(
        &mut self,
        chan_msg: &'a NowVirtualChannel<'msg>,
    ) -> ChannelsManagerResult<'msg> {
        if let Some(sm) = self.state_machines.get_mut(chan_msg.get_name()) {
            sm.update_with_chan_msg(chan_msg)
                .map(|o| o.map(|chan| (sm.get_channel_name(), chan)))
        } else {
            ProtoError::new(ProtoErrorKind::ChannelsManager)
                .or_desc(format!("state machine for channel {:?} not found", chan_msg.get_name()))
        }
    }

    pub fn update_without_virt_msg<'msg>(&mut self) -> ChannelsManagerResult<'msg> {
        for sm in self.state_machines.values_mut() {
            if !sm.waiting_for_packet() {
                return sm
                    .update_without_chan_msg()
                    .map(|o| o.map(|chan| (sm.get_channel_name(), chan)));
            }
        }
        ProtoError::new(ProtoErrorKind::ChannelsManager)
            .or_desc("no channel state machine is ready to update without message")
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
