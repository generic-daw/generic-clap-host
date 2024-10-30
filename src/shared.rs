use crate::MainThreadMessage;
#[cfg(feature = "gui")]
use clack_extensions::gui::{GuiSize, HostGuiImpl};
#[cfg(feature = "params")]
use clack_extensions::params::HostParamsImplShared;
#[cfg(feature = "state")]
use clack_extensions::state::PluginState;
use clack_host::prelude::*;
use std::sync::mpsc::Sender;
#[cfg(feature = "state")]
use std::sync::OnceLock;

pub struct Shared {
    sender: Sender<MainThreadMessage>,
    #[cfg(feature = "state")]
    pub state: OnceLock<Option<PluginState>>,
}

impl<'a> SharedHandler<'a> for Shared {
    fn request_process(&self) {
        // we never pause
    }

    fn request_callback(&self) {
        self.sender
            .send(MainThreadMessage::RunOnMainThread)
            .unwrap();
    }

    fn request_restart(&self) {
        // we don't support restarting plugins (yet)
    }

    #[cfg(feature = "state")]
    fn initializing(&self, instance: InitializingPluginHandle<'a>) {
        #[cfg(feature = "state")]
        self.state.set(instance.get_extension()).ok().unwrap();
    }
}

#[cfg(feature = "gui")]
impl HostGuiImpl for Shared {
    fn resize_hints_changed(&self) {
        // we don't support resize hints (yet)
    }

    fn request_resize(&self, new_size: GuiSize) -> Result<(), HostError> {
        Ok(self
            .sender
            .send(MainThreadMessage::GuiRequestResized(new_size))?)
    }

    fn request_show(&self) -> Result<(), HostError> {
        // we never hide the window, so showing it does nothing
        Ok(())
    }

    fn request_hide(&self) -> Result<(), HostError> {
        // we never hide the window
        Ok(())
    }

    fn closed(&self, _was_destroyed: bool) {
        self.sender.send(MainThreadMessage::GuiClosed).unwrap();
    }
}

#[cfg(feature = "params")]
impl HostParamsImplShared for Shared {
    fn request_flush(&self) {
        // Can never flush events when not processing: we're never not processing
    }
}

impl Shared {
    pub fn new(sender: Sender<MainThreadMessage>) -> Self {
        Self {
            sender,
            #[cfg(feature = "state")]
            state: OnceLock::new(),
        }
    }
}
