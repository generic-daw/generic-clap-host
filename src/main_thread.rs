#[cfg(feature = "timer")]
use crate::extensions::timer::Timers;
#[cfg(feature = "state")]
use crate::shared::Shared;
#[cfg(feature = "audio-ports")]
use clack_extensions::audio_ports::{HostAudioPortsImpl, RescanType};
#[cfg(feature = "gui")]
use clack_extensions::gui::{GuiSize, PluginGui};
#[cfg(feature = "log")]
use clack_extensions::log::{HostLogImpl, LogSeverity};
#[cfg(feature = "note-ports")]
use clack_extensions::note_ports::{HostNotePortsImpl, NoteDialects, NotePortRescanFlags};
#[cfg(feature = "params")]
use clack_extensions::params::{HostParamsImplMainThread, ParamClearFlags, ParamRescanFlags};
#[cfg(feature = "state")]
use clack_extensions::state::HostStateImpl;
#[cfg(feature = "timer")]
use clack_extensions::timer::{HostTimerImpl, PluginTimer, TimerId};
use clack_host::prelude::*;
#[cfg(feature = "timer")]
use std::{rc::Rc, time::Duration};
#[cfg(feature = "log")]
use tracing::{debug, error, info, warn};

pub enum MainThreadMessage {
    RunOnMainThread,
    #[cfg(feature = "gui")]
    GuiClosed,
    #[cfg(feature = "gui")]
    GuiRequestResized(GuiSize),
    ProcessAudio(Vec<Vec<f32>>, AudioPorts, AudioPorts, EventBuffer),
    GetCounter,
    #[cfg(feature = "state")]
    GetState,
    #[cfg(feature = "state")]
    SetState(Vec<u8>),
}

pub struct MainThread<'a> {
    #[cfg(feature = "state")]
    pub shared: &'a Shared,
    plugin: Option<InitializedPluginHandle<'a>>,
    #[cfg(feature = "gui")]
    pub gui: Option<PluginGui>,
    #[cfg(feature = "timer")]
    pub timer_support: Option<PluginTimer>,
    #[cfg(feature = "timer")]
    pub timers: Rc<Timers>,
    #[cfg(feature = "state")]
    pub dirty: bool,
}

#[cfg(not(feature = "state"))]
impl MainThread<'_> {
    pub fn new() -> Self {
        Self {
            plugin: None,
            #[cfg(feature = "gui")]
            gui: None,
            #[cfg(feature = "timer")]
            timer_support: None,
            #[cfg(feature = "timer")]
            timers: Rc::default(),
        }
    }
}

#[cfg(feature = "state")]
impl<'a> MainThread<'a> {
    pub fn new(shared: &'a Shared) -> Self {
        Self {
            shared,
            plugin: None,
            #[cfg(feature = "gui")]
            gui: None,
            #[cfg(feature = "timer")]
            timer_support: None,
            #[cfg(feature = "timer")]
            timers: Rc::default(),
            dirty: false,
        }
    }
}

impl<'a> MainThreadHandler<'a> for MainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        #[cfg(feature = "gui")]
        {
            self.gui = instance.get_extension();
        }
        #[cfg(feature = "timer")]
        {
            self.timer_support = instance.get_extension();
            self.timers = Rc::new(Timers::default());
        }
        self.plugin = Some(instance);
    }
}

#[cfg(feature = "audio-ports")]
impl HostAudioPortsImpl for MainThread<'_> {
    fn is_rescan_flag_supported(&self, _flag: RescanType) -> bool {
        false
    }

    fn rescan(&mut self, _flag: RescanType) {
        // we don't support audio ports changing on the fly (yet)
    }
}

#[cfg(feature = "log")]
impl HostLogImpl for MainThread<'_> {
    fn log(&self, severity: LogSeverity, message: &str) {
        match severity {
            LogSeverity::Info => info!(message),
            LogSeverity::Debug => debug!(message),
            LogSeverity::Warning => warn!(message),
            LogSeverity::Error
            | LogSeverity::Fatal
            | LogSeverity::HostMisbehaving
            | LogSeverity::PluginMisbehaving => error!(message),
        }
    }
}

#[cfg(feature = "note-ports")]
impl HostNotePortsImpl for MainThread<'_> {
    fn supported_dialects(&self) -> NoteDialects {
        NoteDialects::CLAP
    }

    fn rescan(&mut self, _flags: NotePortRescanFlags) {
        // We don't support note ports changing on the fly (yet)
    }
}

#[cfg(feature = "params")]
impl HostParamsImplMainThread for MainThread<'_> {
    fn clear(&mut self, _id: ClapId, _flags: ParamClearFlags) {}

    fn rescan(&mut self, _flags: ParamRescanFlags) {
        // We don't track param values (yet)
    }
}

#[cfg(feature = "state")]
impl HostStateImpl for MainThread<'_> {
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

#[cfg(feature = "timer")]
impl HostTimerImpl for MainThread<'_> {
    fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, HostError> {
        Ok(self
            .timers
            .register_new(Duration::from_millis(u64::from(period_ms))))
    }

    fn unregister_timer(&mut self, timer_id: TimerId) -> Result<(), HostError> {
        if self.timers.unregister(timer_id) {
            Ok(())
        } else {
            Err(HostError::Message("Unknown timer ID"))
        }
    }
}
