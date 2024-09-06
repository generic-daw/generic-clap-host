use crate::MainThreadMessage;
use clack_extensions::gui::{GuiSize, HostGuiImpl};
use clack_host::prelude::*;
use std::sync::mpsc::Sender;

pub struct Shared {
    sender: Sender<MainThreadMessage>,
}

impl<'a> SharedHandler<'a> for Shared {
    fn request_process(&self) {}
    fn request_callback(&self) {}
    fn request_restart(&self) {}
}

impl HostGuiImpl for Shared {
    fn resize_hints_changed(&self) {}

    fn request_resize(&self, new_size: GuiSize) -> Result<(), HostError> {
        Ok(self
            .sender
            .send(MainThreadMessage::GuiRequestResized(new_size))?)
    }

    fn request_show(&self) -> Result<(), HostError> {
        Ok(())
    }

    fn request_hide(&self) -> Result<(), HostError> {
        Ok(())
    }

    fn closed(&self, _was_destroyed: bool) {
        self.sender.send(MainThreadMessage::GuiClosed).unwrap();
    }
}

impl Shared {
    pub const fn new(sender: Sender<MainThreadMessage>) -> Self {
        Self { sender }
    }
}
