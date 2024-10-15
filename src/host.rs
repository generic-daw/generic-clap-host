use crate::{MainThread, Shared};
use clack_host::prelude::*;

#[cfg(feature = "gui")]
use clack_extensions::gui::HostGui;
#[cfg(feature = "params")]
use clack_extensions::params::HostParams;

pub struct Host;

#[derive(Debug)]
pub enum HostThreadMessage {
    AudioProcessed([Vec<f32>; 2], EventBuffer),
    Counter(u64),
}

impl HostHandlers for Host {
    type Shared<'a> = Shared;
    type MainThread<'a> = MainThread<'a>;
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<'_, Self>, _shared: &Self::Shared<'_>) {
        #[cfg(feature = "gui")]
        builder.register::<HostGui>();
        #[cfg(feature = "params")]
        builder.register::<HostParams>();
    }
}
