use crate::prelude::*;
use cpal::{self, traits::DeviceTrait};
use std::{cell::RefCell, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use crate::loader::asset::AudioAsset;

pub struct SysOutputDevice {
    id: u64,
    device: cpal::Device,
    format: cpal::SupportedFormat,
}

impl SysOutputDevice {
    pub fn new(device: cpal::Device) -> Self {
        let id = crate::supervisor::linker::new_id();
        info!("Sys output sample id {}", id);
        let mut formats_range = device
            .supported_output_formats()
            .expect("Wrong output device");
        let format = formats_range
            .next()
            .expect("Output device supported format");
        Self { device, format, id }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.format.max_sample_rate.0
    }

    pub fn get_block_size(&self) -> u32 {
        self.get_sample_rate() / 100 * (self.format.channels as u32)
    }
}

impl SampleDevice for SysOutputDevice {
    fn id(&self) -> u64 {
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

pub struct LoggerSample {
    id: u64,
    buffer: RefCell<Vec<f32>>,
}

impl LoggerSample {
    pub fn new(size: usize) -> Self {
        let id = crate::supervisor::linker::new_id();
        info!("Logger sample id: {}", id);
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

    fn id(&self) -> u64 {
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
    id: u64,
    asset: AudioAsset,
    block_size: usize,
    buffer: Arc<RwLock<Vec<f32>>>,
}

impl AssetSampleOutput {
    pub fn new(asset: AudioAsset, block_size: usize) -> Self {
        let id = crate::supervisor::linker::new_id();
        info!("Asset sample {}", id);
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

    fn id(&self) -> u64 {
        self.id
    }
}

impl SampleOutput for AssetSampleOutput {
    fn next(&self, channel: usize) -> Option<std::sync::RwLockReadGuard<Vec<f32>>> {
        Some(self.buffer.read().unwrap())
    }
}
