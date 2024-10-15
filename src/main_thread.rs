use clack_host::prelude::*;

#[cfg(feature = "gui")]
use clack_extensions::gui::{GuiSize, PluginGui};
#[cfg(feature = "params")]
use clack_extensions::params::{HostParamsImplMainThread, ParamClearFlags, ParamRescanFlags};

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
}

impl<'a> MainThreadHandler<'a> for MainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        #[cfg(feature = "gui")]
        {
            self.gui = instance.get_extension();
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

impl<'a> MainThread<'a> {
    pub fn new() -> Self {
        Self {
            plugin: None,
            #[cfg(feature = "gui")]
            gui: None,
        }
    }
}

impl<'a> Default for MainThread<'a> {
    fn default() -> Self {
        Self::new()
    }
}
