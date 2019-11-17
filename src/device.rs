use crate::linker::{SampleDevice, SampleInput, SampleOutput};
use cpal;
use cpal::traits::DeviceTrait;

pub struct SysOutputDevice {
    id: u64,
    device: cpal::Device,
    format: cpal::SupportedFormat,
}

impl SysOutputDevice {
    pub fn new(device: cpal::Device) -> Self {
        let id = crate::linker::new_id();
        info!("Sys output sample id {}", id);
        let mut formats_range = device
            .supported_output_formats()
            .expect("Wrong output device");
        let format = formats_range
            .next()
            .expect("Output device supported format");
        Self { device, format, id}
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
