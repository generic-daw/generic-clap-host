#[cfg(feature = "timer")]
use crate::extensions::timer::Timers;
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
use clack_extensions::timer::{HostTimerImpl, TimerId};
use clack_host::prelude::*;
#[cfg(feature = "timer")]
use std::{rc::Rc, time::Duration};
#[cfg(feature = "log")]
use tracing::{debug, error, info, warn};

#[expect(missing_debug_implementations)]
pub enum MainThreadMessage {
    RunOnMainThread,
    #[cfg(feature = "gui")]
    GuiClosed,
    #[cfg(feature = "gui")]
    GuiRequestResized(GuiSize),
    ProcessAudio(Vec<Vec<f32>>, AudioPorts, AudioPorts, EventBuffer),
    GetCounter,
}

#[derive(Default)]
pub struct MainThread<'a> {
    plugin: Option<InitializedPluginHandle<'a>>,
    #[cfg(feature = "gui")]
    pub gui: Option<PluginGui>,
    #[cfg(feature = "timer")]
    pub timers: Rc<Timers>,
}

impl<'a> MainThreadHandler<'a> for MainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        #[cfg(feature = "gui")]
        {
            self.gui = instance.get_extension();
        }
        self.plugin = Some(instance);
    }
}

#[cfg(feature = "audio-ports")]
impl HostAudioPortsImpl for MainThread<'_> {
    fn is_rescan_flag_supported(&self, _flag: RescanType) -> bool {
        todo!()
    }

    fn rescan(&mut self, _flag: RescanType) {
        todo!()
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
        todo!()
    }

    fn rescan(&mut self, _flags: NotePortRescanFlags) {
        todo!()
    }
}

#[cfg(feature = "params")]
impl HostParamsImplMainThread for MainThread<'_> {
    fn clear(&mut self, _id: ClapId, _flags: ParamClearFlags) {
        todo!()
    }

    fn rescan(&mut self, _flags: ParamRescanFlags) {
        todo!()
    }
}

#[cfg(feature = "state")]
impl HostStateImpl for MainThread<'_> {
    fn mark_dirty(&mut self) {
        todo!()
    }
}

#[cfg(feature = "timer")]
impl<'a> HostTimerImpl for MainThread<'a> {
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
