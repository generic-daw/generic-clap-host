pub mod audio_processor;
mod extensions;
pub mod host;
pub mod main_thread;
mod shared;

use audio_processor::AudioProcessor;
use clack_host::prelude::*;
use extensions::gui::Gui;
use host::{Host, HostThreadMessage};
use main_thread::{MainThread, MainThreadMessage};
use shared::Shared;
use std::{
    path::PathBuf,
    result::Result,
    sync::mpsc::{Receiver, Sender},
};
use walkdir::WalkDir;

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
    let mut paths = vec![];

    if let Some(home_dir) = dirs::home_dir() {
        paths.push(home_dir.join(".clap"));

        #[cfg(target_os = "macos")]
        {
            paths.push(home_dir.join("Library/Audio/Plug-Ins/CLAP"));
        }
    }

    #[cfg(windows)]
    {
        if let Some(val) = std::env::var_os("CommonProgramFiles") {
            paths.push(PathBuf::from(val).join("CLAP"));
        }

        if let Some(dir) = dirs::config_local_dir() {
            paths.push(dir.join("Programs\\Common\\CLAP"));
        }
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

        let mut gui = instance
            .access_handler(|h| h.gui)
            .map(|gui| Gui::new(gui, &mut instance.plugin_handle()))
            .unwrap();

        if gui.needs_floating().unwrap() {
            gui.run_gui_floating(
                instance,
                &sender_host,
                receiver_plugin,
                &mut AudioProcessor::new(audio_processor),
            );
        } else {
            gui.run_gui_embedded(
                instance,
                &sender_host,
                receiver_plugin,
                &mut AudioProcessor::new(audio_processor),
            );
        }
    });

    (sender_plugin, receiver_host)
}
