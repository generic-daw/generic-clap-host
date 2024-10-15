pub(crate) mod audio_processor;
mod extensions;

pub(crate) mod host;
pub use host::HostThreadMessage;

pub(crate) mod main_thread;
pub use main_thread::MainThreadMessage;

mod shared;

pub use clack_host;

use audio_processor::AudioProcessor;
use clack_host::prelude::*;
use etcetera::{choose_base_strategy, BaseStrategy};
use host::Host;
use main_thread::MainThread;
use shared::Shared;
use std::{
    path::PathBuf,
    result::Result,
    sync::mpsc::{Receiver, Sender},
};
use walkdir::WalkDir;

#[cfg(feature = "gui")]
use extensions::gui::GuiExt;

#[cfg(not(feature = "gui"))]
use extensions::no_gui::run_no_gui;

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

    #[cfg(target_os = "macos")]
    {
        paths.push(strategy.home_dir().join("Library/Audio/Plug-Ins/CLAP"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(val) = std::env::var_os("CommonProgramFiles") {
            paths.push(PathBuf::from(val).join("CLAP"));
        }

        paths.push(strategy.config_dir().join("Programs\\Common\\CLAP"));
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));
    }

    #[cfg(target_family = "unix")]
    {
        paths.push("/usr/lib/clap".into());
    }

    if let Some(env_var) = std::env::var_os("CLAP_PATH") {
        paths.extend(std::env::split_paths(&env_var));
    }

    paths
}

/// # Panics
///
/// panics if the plugin doesn't expose a `PluginFactory`
#[must_use]
pub fn run(
    bundle: PluginBundle,
    config: PluginAudioConfiguration,
) -> (Sender<MainThreadMessage>, Receiver<HostThreadMessage>) {
    let (sender_plugin, receiver_plugin) = std::sync::mpsc::channel();
    let (sender_host, receiver_host) = std::sync::mpsc::channel();

    let sender_plugin_clone = sender_plugin.clone();
    std::thread::spawn(move || {
        let factory = bundle.get_plugin_factory().unwrap();
        let plugin_descriptor = factory.plugin_descriptors().next().unwrap();
        let mut instance = PluginInstance::<Host>::new(
            |()| Shared::new(sender_plugin_clone),
            |_| MainThread::new(),
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
        {
            run_no_gui(
                instance,
                &sender_host,
                receiver_plugin,
                &mut AudioProcessor::new(audio_processor, config),
            );
        }

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
                    receiver_plugin,
                    &mut AudioProcessor::new(audio_processor, config),
                );
            } else {
                gui.run_gui_embedded(
                    instance,
                    &sender_host,
                    receiver_plugin,
                    &mut AudioProcessor::new(audio_processor, config),
                );
            }
        }
    });

    (sender_plugin, receiver_host)
}
