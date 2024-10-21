use crate::{AudioProcessor, Host, HostThreadMessage, MainThreadMessage};
use clack_host::plugin::PluginInstance;
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
            }
        }

        std::thread::sleep(
            #[cfg(feature = "timer")]
            {
                if let Some((timers, timer_ext)) = &timers {
                    timers.tick_timers(timer_ext, &mut instance.plugin_handle());

                    timers
                        .smallest_duration()
                        .unwrap_or(Duration::from_millis(30))
                } else {
                    Duration::from_millis(30)
                }
            },
            #[cfg(not(feature = "timer"))]
            {
                Duration::from_millis(30)
            },
        );
    }
}
