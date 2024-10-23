use crate::{AudioProcessor, Host, HostThreadMessage, MainThreadMessage};
use clack_extensions::gui::{GuiApiType, GuiConfiguration, GuiError, GuiSize, PluginGui};
use clack_host::prelude::*;
#[cfg(feature = "timer")]
use std::time::Instant;
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};
use winit::dpi::{LogicalSize, PhysicalSize, Size};

pub struct GuiExt {
    plugin_gui: PluginGui,
    pub configuration: Option<GuiConfiguration<'static>>,
    is_open: bool,
    is_resizeable: bool,
}

impl GuiExt {
    pub fn new(plugin_gui: PluginGui, instance: &mut PluginMainThreadHandle<'_>) -> Self {
        Self {
            plugin_gui,
            configuration: Self::negotiate_configuration(&plugin_gui, instance),
            is_open: false,
            is_resizeable: false,
        }
    }

    fn negotiate_configuration(
        gui: &PluginGui,
        plugin: &mut PluginMainThreadHandle<'_>,
    ) -> Option<GuiConfiguration<'static>> {
        let api_type = GuiApiType::default_for_current_platform()?;
        let mut config = GuiConfiguration {
            api_type,
            is_floating: false,
        };

        if gui.is_api_supported(plugin, config) {
            Some(config)
        } else {
            config.is_floating = true;
            if gui.is_api_supported(plugin, config) {
                Some(config)
            } else {
                None
            }
        }
    }

    pub fn gui_size_to_winit_size(&self, size: GuiSize) -> Size {
        let Some(GuiConfiguration { api_type, .. }) = self.configuration else {
            panic!("Called gui_size_to_winit_size on incompatible plugin")
        };

        if api_type.uses_logical_size() {
            LogicalSize {
                width: size.width,
                height: size.height,
            }
            .into()
        } else {
            PhysicalSize {
                width: size.width,
                height: size.height,
            }
            .into()
        }
    }

    pub fn needs_floating(&self) -> Option<bool> {
        self.configuration
            .map(|GuiConfiguration { is_floating, .. }| is_floating)
    }

    pub fn open_floating(&self, plugin: &mut PluginMainThreadHandle<'_>) -> Result<(), GuiError> {
        let Some(configuration) = self.configuration else {
            panic!("Called open_floating on incompatible plugin")
        };
        assert!(
            configuration.is_floating,
            "Called open_floating on incompatible plugin"
        );

        self.plugin_gui.create(plugin, configuration)?;
        self.plugin_gui.suggest_title(plugin, c"");
        self.plugin_gui.show(plugin)?;

        Ok(())
    }

    pub fn resize(
        &self,
        plugin: &mut PluginMainThreadHandle<'_>,
        size: Size,
        scale_factor: f64,
    ) -> Size {
        let uses_logical_pixels = self.configuration.unwrap().api_type.uses_logical_size();

        let size = if uses_logical_pixels {
            let size = size.to_logical(scale_factor);
            GuiSize {
                width: size.width,
                height: size.height,
            }
        } else {
            let size = size.to_physical(scale_factor);
            GuiSize {
                width: size.width,
                height: size.height,
            }
        };

        if !self.is_resizeable {
            let forced_size = self.plugin_gui.get_size(plugin).unwrap_or(size);

            return self.gui_size_to_winit_size(forced_size);
        }

        let working_size = self.plugin_gui.adjust_size(plugin, size).unwrap_or(size);
        self.plugin_gui.set_size(plugin, working_size).unwrap();

        self.gui_size_to_winit_size(working_size)
    }

    pub fn destroy(&mut self, plugin: &mut PluginMainThreadHandle<'_>) {
        if self.is_open {
            self.plugin_gui.destroy(plugin);
            self.is_open = false;
        }
    }

    #[expect(clippy::needless_pass_by_ref_mut)]
    pub fn run_gui_embedded(
        &mut self,
        mut _instance: PluginInstance<Host>,
        _sender: &Sender<HostThreadMessage>,
        _receiver: &Receiver<MainThreadMessage>,
        _audio_processor: &mut AudioProcessor,
    ) {
        todo!()
    }

    pub fn run_gui_floating(
        &mut self,
        mut instance: PluginInstance<Host>,
        sender: &Sender<HostThreadMessage>,
        receiver: &Receiver<MainThreadMessage>,
        audio_processor: &mut AudioProcessor,
    ) {
        self.open_floating(&mut instance.plugin_handle()).unwrap();
        #[cfg(feature = "timer")]
        let timers =
            instance.access_handler(|h| h.timer_support.map(|ext| (h.timers.clone(), ext)));

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
                    MainThreadMessage::GuiClosed { .. } => {
                        self.destroy(&mut instance.plugin_handle());
                        return;
                    }
                    MainThreadMessage::GuiRequestResized(gui_size) => {
                        self.resize(
                            &mut instance.plugin_handle(),
                            self.gui_size_to_winit_size(gui_size),
                            1.0f64,
                        );
                    }
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
        }
    }
}
