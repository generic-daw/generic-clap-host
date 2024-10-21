use crate::MainThreadMessage;
#[cfg(feature = "gui")]
use clack_extensions::gui::{GuiSize, HostGuiImpl};
#[cfg(feature = "params")]
use clack_extensions::params::HostParamsImplShared;
use clack_host::prelude::*;
use std::sync::mpsc::Sender;

pub struct Shared {
    sender: Sender<MainThreadMessage>,
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
        Self { sender }
    }
}
