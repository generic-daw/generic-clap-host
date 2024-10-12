use crate::{
    audio_processor::AudioProcessor,
    host::{Host, HostThreadMessage},
    main_thread::MainThreadMessage,
};
use clack_host::plugin::PluginInstance;
use std::sync::mpsc::{Receiver, Sender};

pub fn run_no_gui(
    mut instance: PluginInstance<Host>,
    sender: &Sender<HostThreadMessage>,
    receiver: Receiver<MainThreadMessage>,
    audio_processor: &mut AudioProcessor,
) {
    for message in receiver {
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
            _ => {}
        }
    }
}
