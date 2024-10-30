use crate::{AudioProcessor, Host, HostThreadMessage, MainThreadMessage};
#[cfg(feature = "state")]
use clack_extensions::state::PluginState;
use clack_host::prelude::*;
#[cfg(feature = "state")]
use std::io::Cursor;
#[cfg(feature = "timer")]
use std::time::Instant;
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

pub fn run_no_gui(
    mut instance: PluginInstance<Host>,
    sender: &Sender<HostThreadMessage>,
    receiver: &Receiver<MainThreadMessage>,
    audio_processor: &mut AudioProcessor,
) {
    #[cfg(feature = "timer")]
    let timers = instance.access_handler(|h| h.timer_support.map(|ext| (h.timers.clone(), ext)));

    loop {
        #[cfg(feature = "timer")]
        let sleep_duration = timers
            .as_ref()
            .and_then(|(timers, _)| Some(timers.next_tick()? - Instant::now()))
            .unwrap_or(Duration::from_millis(30))
            .min(Duration::from_millis(30));
        #[cfg(not(feature = "timer"))]
        let sleep_duration = Duration::from_millis(30);

        std::thread::sleep(sleep_duration);

        #[cfg(feature = "timer")]
        if let Some((timers, timer_ext)) = &timers {
            timers.tick_timers(timer_ext, &mut instance.plugin_handle());
        }

        while let Ok(message) = receiver.try_recv() {
            match message {
                MainThreadMessage::RunOnMainThread => instance.call_on_main_thread_callback(),
                MainThreadMessage::ProcessAudio(
                    mut input_buffers,
                    mut input_audio_ports,
                    mut output_audio_ports,
                    input_events,
                ) => {
                    let (output_buffers, output_events) = audio_processor.process(
                        &mut input_buffers,
                        &input_events,
                        &mut input_audio_ports,
                        &mut output_audio_ports,
                    );

                    sender
                        .send(HostThreadMessage::AudioProcessed(
                            output_buffers,
                            output_events,
                        ))
                        .unwrap();
                }
                MainThreadMessage::GetCounter => {
                    sender
                        .send(HostThreadMessage::Counter(audio_processor.steady_time()))
                        .unwrap();
                }
                #[cfg(feature = "state")]
                MainThreadMessage::GetState => {
                    let state_ext: PluginState = instance
                        .access_handler_mut(|h| h.shared.state.get())
                        .unwrap()
                        .unwrap();

                    let mut state = Vec::new();
                    state_ext
                        .save(&mut instance.plugin_handle(), &mut state)
                        .unwrap();

                    sender.send(HostThreadMessage::State(state)).unwrap();
                }
                #[cfg(feature = "state")]
                MainThreadMessage::SetState(state) => {
                    let state_ext: PluginState = instance
                        .access_handler_mut(|h| h.shared.state.get())
                        .unwrap()
                        .unwrap();

                    let mut state = Cursor::new(state);

                    state_ext
                        .load(&mut instance.plugin_handle(), &mut state)
                        .unwrap();
                }
            }
        }
    }
}
