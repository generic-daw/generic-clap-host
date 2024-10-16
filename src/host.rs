use crate::{MainThread, Shared};
#[cfg(feature = "audio-ports")]
use clack_extensions::audio_ports::HostAudioPorts;
#[cfg(feature = "gui")]
use clack_extensions::gui::HostGui;
#[cfg(feature = "note-ports")]
use clack_extensions::note_ports::HostNotePorts;
#[cfg(feature = "params")]
use clack_extensions::params::HostParams;
#[cfg(feature = "state")]
use clack_extensions::state::HostState;
#[cfg(feature = "timer")]
use clack_extensions::timer::HostTimer;
use clack_host::prelude::*;

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
        #[cfg(feature = "audio-ports")]
        builder.register::<HostAudioPorts>();
        #[cfg(feature = "gui")]
        builder.register::<HostGui>();
        #[cfg(feature = "note-ports")]
        builder.register::<HostNotePorts>();
        #[cfg(feature = "params")]
        builder.register::<HostParams>();
        #[cfg(feature = "state")]
        builder.register::<HostState>();
        #[cfg(feature = "timer")]
        builder.register::<HostTimer>();
        let _ = builder;
    }
}
