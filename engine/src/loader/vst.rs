use crate::{prelude::*, supervisor::linker::Linker, devices::VstBufferedDevice};
use std::{
    ffi::c_void,
    sync::{Arc, RwLock},
};
use vst::{
    api::{self, TimeInfo},
    buffer::AudioBuffer,
    editor::Editor,
    host::{Host, PluginInstance},
    plugin::{Info, Plugin},
};

/// Unique id assigned to a vst instance
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct VstId(u64);

/// VST plugin host
pub struct VstHost {
    pub time_info: Option<TimeInfo>,
    pub block_size: isize,
}

impl VstHost {
    /// Create an empty host
    ///
    /// # Parametters
    /// * `block_size` The default samples block size
    pub fn new(block_size: isize) -> Self {
        Self {
            time_info: None,
            block_size,
        }
    }
}

impl Host for VstHost {
    fn automate(&self, index: i32, value: f32) {
        info!("Parameter {} had its value changed to {}", index, value);
    }
    fn process_events(&self, events: &api::Events) {
        info!("Events: {:?}", events.num_events);
    }

    fn idle(&self) {
        info!("Idle");
    }

    fn get_time_info(&self, mask: i32) -> Option<TimeInfo> {
        Some(TimeInfo::default())
    }

    fn get_block_size(&self) -> isize {
        info!("Return bszie ...");
        self.block_size
    }
}

/// VST Instance wrapper that contains extra informations like I/O devices index
pub struct VstPlugin {
    /// Unique instance id
    pub id: VstId,
    /// VST Basic informations
    info: Info,
    /// VST Instance
    instance: PluginInstance,
    /// Input device (allocated in the linker arena)
    input: InputIndex,
    /// Input output (allocated in the linker arena)
    output: OutputIndex,
}

impl VstPlugin {
    pub fn init(
        mut instance: PluginInstance,
        sample_rate: f32,
        block_size: i64,
        linker: &mut Linker,
    ) -> Self {
        let info = instance.get_info();
        instance.init();
        instance.set_sample_rate(sample_rate);
        instance.set_block_size(block_size);
        instance.resume();
        let id = VstId(crate::supervisor::linker::new_id());
        let virt_device = Box::new(VstBufferedDevice::new(block_size as usize, 2, id));
        let input = linker.register_input(virt_device.clone());
        let output = linker.register_output(virt_device);
        info!("Plugin initialized: {:?}", info);
        Self {
            id,
            info,
            instance,
            input,
            output,
        }
    }

    pub fn get_inputs(&self) -> InputIndex {
        self.input
    }

    pub fn get_outputs(&self) -> OutputIndex {
        self.output
    }

    pub fn load_editor(&mut self, win_handle: *mut c_void) {
        let edit = self.instance.get_editor().expect("Editor");
        info!("Editor size: W {}, H {}", edit.size().0, edit.size().1);
        self.instance.open_editor(win_handle);
    }

    pub fn next<'a>(&mut self, buffer: &mut AudioBuffer<'a, f32>) {
        self.instance.process(buffer);
    }
}
