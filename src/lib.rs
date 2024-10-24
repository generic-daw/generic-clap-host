use audio_processor::AudioProcessor;
pub use clack_host;
use clack_host::prelude::*;
use etcetera::{choose_base_strategy, BaseStrategy};
#[cfg(feature = "gui")]
use extensions::gui::GuiExt;
#[cfg(not(feature = "gui"))]
use extensions::no_gui::run_no_gui;
use host::{Host, HostThreadMessage};
use main_thread::{MainThread, MainThreadMessage};
use shared::Shared;
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    path::PathBuf,
    result::Result,
    sync::mpsc::{Receiver, Sender},
};
use walkdir::WalkDir;

pub(crate) mod audio_processor;
mod extensions;
pub(crate) mod host;
pub(crate) mod main_thread;

mod shared;

#[derive(Debug)]
pub struct ClapPlugin {
    sender: Sender<MainThreadMessage>,
    receiver: Receiver<HostThreadMessage>,
    _no_sync: PhantomData<UnsafeCell<()>>,
}

impl ClapPlugin {
    fn new(sender: Sender<MainThreadMessage>, receiver: Receiver<HostThreadMessage>) -> Self {
        Self {
            sender,
            receiver,
            _no_sync: PhantomData,
        }
    }

    #[must_use]
    /// # Panics
    ///
    /// This  will never panic, since this function blocks until the audio is processed, and you can't send the `ClapPlugin` to another thread.
    pub fn process_audio(
        &self,
        input_audio: Vec<Vec<f32>>,
        input_audio_ports: AudioPorts,
        output_audio_ports: AudioPorts,
        input_events: EventBuffer,
    ) -> (Vec<Vec<f32>>, EventBuffer) {
        self.sender
            .send(MainThreadMessage::ProcessAudio(
                input_audio,
                input_audio_ports,
                output_audio_ports,
                input_events,
            ))
            .unwrap();

        match self.receiver.recv() {
            Ok(HostThreadMessage::AudioProcessed(output_audio, output_events)) => {
                (output_audio, output_events)
            }
            _ => unreachable!(),
        }
    }

    #[must_use]
    /// # Panics
    ///
    /// This  will never panic, since this function blocks until the audio is processed, and you can't send the `ClapPlugin` to another thread.
    pub fn get_counter(&self) -> u64 {
        self.sender.send(MainThreadMessage::GetCounter).unwrap();

        match self.receiver.recv() {
            Ok(HostThreadMessage::Counter(counter)) => counter,
            _ => unreachable!(),
        }
    }
}

#[must_use]
pub fn get_installed_plugins() -> Vec<PluginBundle> {
    standard_clap_paths()
        .iter()
        .flat_map(|path| {
            WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|dir_entry| dir_entry.file_type().is_file())
                .filter(|dir_entry| {
                    dir_entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext == "clap")
                })
        })
        .filter_map(|path| unsafe { PluginBundle::load(path.path()) }.ok())
        .filter(|bundle| {
            bundle
                .get_plugin_factory()
                .is_some_and(|factory| factory.plugin_descriptors().next().is_some())
        })
        .collect()
}

fn standard_clap_paths() -> Vec<PathBuf> {
    let strategy = choose_base_strategy().unwrap();

    let mut paths = vec![];

    paths.push(strategy.home_dir().join(".clap"));

    #[cfg(target_os = "windows")]
    {
        if let Some(val) = std::env::var_os("CommonProgramFiles") {
            paths.push(PathBuf::from(val).join("CLAP"));
        }

        paths.push(strategy.config_dir().join("Programs\\Common\\CLAP"));
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(strategy.home_dir().join("Library/Audio/Plug-Ins/CLAP"));

        paths.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));
    }

    #[cfg(target_family = "unix")]
    paths.push("/usr/lib/clap".into());

    if let Some(env_var) = std::env::var_os("CLAP_PATH") {
        paths.extend(std::env::split_paths(&env_var));
    }

    paths
}

/// # Panics
///
/// panics if the plugin doesn't expose a `PluginFactory`
#[must_use]
pub fn run(bundle: PluginBundle, config: PluginAudioConfiguration) -> ClapPlugin {
    let (sender_plugin, receiver_plugin) = std::sync::mpsc::channel();
    let (sender_host, receiver_host) = std::sync::mpsc::channel();

    let sender_plugin_clone = sender_plugin.clone();

    std::thread::spawn(move || {
        let factory = bundle.get_plugin_factory().unwrap();
        let plugin_descriptor = factory.plugin_descriptors().next().unwrap();
        let mut instance = PluginInstance::<Host>::new(
            |()| Shared::new(sender_plugin_clone),
            |_| MainThread::default(),
            &bundle,
            plugin_descriptor.id().unwrap(),
            &HostInfo::new("", "", "", "").unwrap(),
        )
        .unwrap();

        let audio_processor = instance
            .activate(|_, _| {}, config)
            .unwrap()
            .start_processing()
            .unwrap();

        #[cfg(not(feature = "gui"))]
        run_no_gui(
            instance,
            &sender_host,
            &receiver_plugin,
            &mut AudioProcessor::new(audio_processor, config),
        );

        #[cfg(feature = "gui")]
        {
            let mut gui = instance
                .access_handler(|h| h.gui)
                .map(|gui| GuiExt::new(gui, &mut instance.plugin_handle()))
                .unwrap();

            if gui.needs_floating().unwrap() {
                gui.run_gui_floating(
                    instance,
                    &sender_host,
                    &receiver_plugin,
                    &mut AudioProcessor::new(audio_processor, config),
                );
            } else {
                gui.run_gui_embedded(
                    instance,
                    &sender_host,
                    &receiver_plugin,
                    &mut AudioProcessor::new(audio_processor, config),
                );
            }
        }
    });

    ClapPlugin::new(sender_plugin, receiver_host)
}
