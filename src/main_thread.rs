use clack_extensions::gui::{GuiSize, PluginGui};
use clack_host::prelude::*;

#[expect(missing_debug_implementations)]
pub enum MainThreadMessage {
    RunOnMainThread,
    GuiClosed,
    GuiRequestResized(GuiSize),
    ProcessAudio([Vec<f32>; 2], AudioPorts, AudioPorts, EventBuffer),
    GetCounter,
}

pub struct MainThread<'a> {
    plugin: Option<InitializedPluginHandle<'a>>,
    pub gui: Option<PluginGui>,
}

impl<'a> MainThreadHandler<'a> for MainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        self.gui = instance.get_extension();
        self.plugin = Some(instance);
    }
}

impl<'a> MainThread<'a> {
    pub fn new() -> Self {
        Self {
            plugin: None,
            gui: None,
        }
    }
}

impl<'a> Default for MainThread<'a> {
    fn default() -> Self {
        Self::new()
    }
}
