use clack_host::prelude::*;

#[cfg(feature = "gui")]
use clack_extensions::gui::{GuiSize, PluginGui};
#[cfg(feature = "params")]
use clack_extensions::params::{HostParamsImplMainThread, ParamClearFlags, ParamRescanFlags};
#[cfg(feature = "state")]
use clack_extensions::state::HostStateImpl;

#[expect(missing_debug_implementations)]
pub enum MainThreadMessage {
    RunOnMainThread,
    #[cfg(feature = "gui")]
    GuiClosed,
    #[cfg(feature = "gui")]
    GuiRequestResized(GuiSize),
    ProcessAudio([Vec<f32>; 2], AudioPorts, AudioPorts, EventBuffer),
    GetCounter,
}

pub struct MainThread<'a> {
    plugin: Option<InitializedPluginHandle<'a>>,
    #[cfg(feature = "gui")]
    pub gui: Option<PluginGui>,
    #[cfg(feature = "state")]
    state_dirty: bool,
}

impl<'a> MainThreadHandler<'a> for MainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        #[cfg(feature = "gui")]
        {
            self.gui = instance.get_extension();
        }
        #[cfg(feature = "params")]
        {
            self.state_dirty = false;
        }
        self.plugin = Some(instance);
    }
}

#[cfg(feature = "params")]
impl HostParamsImplMainThread for MainThread<'_> {
    fn clear(&mut self, _id: ClapId, _flags: ParamClearFlags) {
        todo!()
    }

    fn rescan(&mut self, _flags: ParamRescanFlags) {
        todo!()
    }
}

#[cfg(feature = "state")]
impl HostStateImpl for MainThread<'_> {
    fn mark_dirty(&mut self) {
        self.state_dirty = true;
    }
}

impl<'a> MainThread<'a> {
    pub fn new() -> Self {
        Self {
            plugin: None,
            #[cfg(feature = "gui")]
            gui: None,
            #[cfg(feature = "state")]
            state_dirty: false,
        }
    }
}

impl<'a> Default for MainThread<'a> {
    fn default() -> Self {
        Self::new()
    }
}
