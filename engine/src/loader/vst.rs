use crate::prelude::*;
use crate::supervisor::linker::Linker;
use std::{
    ffi::c_void,
    sync::{Arc, RwLock},
};
use vst::{
    buffer::AudioBuffer,
    editor::Editor,
    host::{Host, HostBuffer, PluginInstance, PluginLoader},
    plugin::{Info, Plugin},
    api::{self, TimeInfo},
};

pub struct VstId(u64);

pub struct VstHost {
    pub time_info: Option<TimeInfo>,
    pub block_size: isize,
}

impl VstHost {
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

pub struct VstPlugin {
    pub id: u64,
    info: Info,
    instance: PluginInstance,
    inputs: InputIndex,
    outputs: OutputIndex,
}

#[derive(Clone)]
pub struct VstBufferedDevice {
    id: u64,
    vst_id: u64,
    buffer: Vec<Arc<RwLock<Vec<f32>>>>,
}

impl VstBufferedDevice {
    pub fn new(size: usize, channels: usize, vst_id: u64) -> Self {
        let id = crate::supervisor::linker::new_id();
        info!("VST sample id {}", id);
        Self {
            vst_id,
            id,
            buffer: vec![Arc::new(RwLock::new(vec![0f32; size])); channels],
        }
    }
}

impl SampleDevice for VstBufferedDevice {
    fn block_size(&self) -> usize {
        self.buffer[0].read().unwrap().len()
    }

    fn nbr_channel(&self) -> usize {
        self.buffer.len()
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn parent_vst(&self) -> Option<u64> {
        Some(self.vst_id)
    }
}

impl SampleOutput for VstBufferedDevice {
    fn next(&self, channel: usize) -> Option<std::sync::RwLockReadGuard<Vec<f32>>> {
        Some(self.buffer[channel].read().unwrap())
    }
}

impl SampleInput for VstBufferedDevice {
    fn next(&mut self, buffer: &[f32], channel: usize) {
        self.buffer[channel]
            .write()
            .unwrap()
            .copy_from_slice(buffer);
    }
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
        let id = crate::supervisor::linker::new_id();
        let virt_device = Box::new(VstBufferedDevice::new(block_size as usize, 2, id));
        let inputs = linker.register_input(virt_device.clone());
        let outputs = linker.register_output(virt_device);
        info!("Plugin initialized: {:?}", info);
        Self {
            id,
            info,
            instance,
            inputs,
            outputs,
        }
    }

    pub fn get_inputs(&self) -> InputIndex {
        self.inputs.clone()
    }

    pub fn get_outputs(&self) -> OutputIndex {
        self.outputs.clone()
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
