use crate::{main_thread::MainThread, shared::Shared};
use clack_extensions::gui::HostGui;
use clack_host::prelude::*;

pub(crate) struct Host;

pub enum HostThreadMessage {
    AudioProcessed([Vec<f32>; 2], EventBuffer),
    Counter(u64),
}

impl HostHandlers for Host {
    type Shared<'a> = Shared;
    type MainThread<'a> = MainThread<'a>;
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<'_, Self>, _shared: &Self::Shared<'_>) {
        builder.register::<HostGui>();
    }
}
