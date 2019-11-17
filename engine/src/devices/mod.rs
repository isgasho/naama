use crate::loader::asset::AudioAsset;
use crate::prelude::*;
use cpal::{self, traits::DeviceTrait};
use std::{
    cell::RefCell,
    sync::{Arc, RwLock, RwLockReadGuard},
};

/// Output samples to a system device (sound card [...])
pub struct SysOutputDevice {
    id: DeviceId,
    device: cpal::Device,
    format: cpal::SupportedFormat,
}

impl SysOutputDevice {
    /// Create a new output device
    ///
    /// # Parametters
    /// * `device` The cpal device to write into
    pub fn new(device: cpal::Device) -> Self {
        let id = DeviceId(crate::supervisor::linker::new_id());
        let mut formats_range = device
            .supported_output_formats()
            .expect("Wrong output device");
        let format = formats_range
            .next()
            .expect("Output device supported format");
        Self { device, format, id }
    }

    /// Get the device sample rate
    pub fn get_sample_rate(&self) -> u32 {
        self.format.max_sample_rate.0
    }

    /// Get the device samples block size
    pub fn get_block_size(&self) -> u32 {
        self.get_sample_rate() / 100 * (self.format.channels as u32)
    }
}

impl SampleDevice for SysOutputDevice {
    fn id(&self) -> DeviceId {
        self.id
    }
    fn block_size(&self) -> usize {
        self.get_block_size() as usize
    }
    fn nbr_channel(&self) -> usize {
        2
    }
}

impl SampleOutput for SysOutputDevice {}

/// Black hole that just log incoming samples
pub struct LoggerSample {
    id: DeviceId,
    buffer: RefCell<Vec<f32>>,
}

impl LoggerSample {
    /// Create a new logger
    ///
    /// # Parametters
    /// * `size` size of the sample buffer
    pub fn new(size: usize) -> Self {
        let id = DeviceId(crate::supervisor::linker::new_id());
        Self {
            id,
            buffer: RefCell::new(vec![0f32; size]),
        }
    }
}

impl SampleDevice for LoggerSample {
    fn block_size(&self) -> usize {
        self.buffer.borrow().len()
    }

    fn nbr_channel(&self) -> usize {
        2
    }

    fn id(&self) -> DeviceId {
        self.id
    }
}

impl SampleInput for LoggerSample {
    fn next(&mut self, buffer: &[f32], channel: usize) {
        info!("Final output {:?}", &buffer[0..4]);
    }
}

#[derive(Debug)]
pub struct AssetSampleOutput {
    id: DeviceId,
    asset: AudioAsset,
    block_size: usize,
    buffer: Arc<RwLock<Vec<f32>>>,
}

impl AssetSampleOutput {
    pub fn new(asset: AudioAsset, block_size: usize) -> Self {
        let id = DeviceId(crate::supervisor::linker::new_id());
        let buffer = Arc::new(RwLock::new(Vec::from(&asset.buffer[0..block_size])));
        Self {
            id,
            block_size,
            asset,
            buffer,
        }
    }
}

impl SampleDevice for AssetSampleOutput {
    fn block_size(&self) -> usize {
        self.block_size
    }

    fn nbr_channel(&self) -> usize {
        2
    }

    fn id(&self) -> DeviceId {
        self.id
    }
}

impl SampleOutput for AssetSampleOutput {
    fn next(&self, channel: usize) -> Option<RwLockReadGuard<Vec<f32>>> {
        Some(self.buffer.read().unwrap())
    }
}

/// A device that send or get data from/to a loaded vst plugin
#[derive(Clone)]
pub struct VstBufferedDevice {
    id: DeviceId,
    vst_id: VstId,
    buffer: Vec<Arc<RwLock<Vec<f32>>>>,
}

impl VstBufferedDevice {
    pub fn new(size: usize, channels: usize, vst_id: VstId) -> Self {
        let id = DeviceId(crate::supervisor::linker::new_id());
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

    fn id(&self) -> DeviceId {
        self.id
    }

    fn parent_vst(&self) -> Option<VstId> {
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
