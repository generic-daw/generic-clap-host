use std::sync::{Arc, RwLock};

use clack_extensions::gui::{GuiSize, PluginGui};
use clack_host::prelude::*;

pub enum MainThreadMessage {
    RunOnMainThread,
    GuiClosed,
    GuiRequestResized(GuiSize),
    ProcessAudio(
        [Vec<f32>; 2],
        Arc<RwLock<AudioPorts>>,
        Arc<RwLock<AudioPorts>>,
        EventBuffer,
    ),
    GetCounter,
}

pub(crate) struct MainThread<'a> {
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
    pub const fn new() -> Self {
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
