#[cfg(feature = "gui")]
use crate::MainThreadMessage;
#[cfg(feature = "gui")]
use clack_extensions::gui::{GuiSize, HostGuiImpl};
#[cfg(feature = "params")]
use clack_extensions::params::HostParamsImplShared;
use clack_host::prelude::*;
#[cfg(feature = "gui")]
use std::sync::mpsc::Sender;

pub struct Shared {
    #[cfg(feature = "gui")]
    sender: Sender<MainThreadMessage>,
}

impl<'a> SharedHandler<'a> for Shared {
    fn request_process(&self) {}
    fn request_callback(&self) {}
    fn request_restart(&self) {}
}

#[cfg(feature = "gui")]
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

#[cfg(feature = "params")]
impl HostParamsImplShared for Shared {
    fn request_flush(&self) {
        todo!()
    }
}

impl Shared {
    pub fn new(#[cfg(feature = "gui")] sender: Sender<MainThreadMessage>) -> Self {
        Self {
            #[cfg(feature = "gui")]
            sender,
        }
    }
}
