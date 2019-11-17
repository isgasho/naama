use crate::linker::{SampleDevice, SampleOutput};
use claxon::FlacReader;
use sample::conv;
use std::io::Read;

#[derive(Debug, Fail)]
pub enum AssetError {
    #[fail(display = "FLAC Decoding error: {:?}", 0)]
    FLACDecoding(claxon::Error),
}

impl From<claxon::Error> for AssetError {
    fn from(err: claxon::Error) -> Self {
        AssetError::FLACDecoding(err)
    }
}

#[derive(Debug, Clone)]
pub struct AudioAsset {
    pub buffer: Vec<f32>,
}

impl AudioAsset {
    pub fn from_flac_file<T: Read>(rd: T) -> Result<Self, AssetError> {
        let mut reader = FlacReader::new(rd)?;
        let mut buffer = Vec::new(); //TODO pre-alocate enough size to fill all samples
        for sample in reader.samples() {
            let sample = conv::i32::to_f32(sample.unwrap());
            buffer.push(sample);
        }
        Ok(AudioAsset { buffer })
    }
}

use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct AssetSampleOutput {
    id: u64,
    asset: AudioAsset,
    block_size: usize,
    buffer: Arc<RwLock<Vec<f32>>>,
}

impl AssetSampleOutput {
    pub fn new(asset: AudioAsset, block_size: usize) -> Self {
        let id = crate::linker::new_id();
        info!("Asset sample {}", id); 
        let buffer = Arc::new(RwLock::new(Vec::from(&asset.buffer[0..block_size])));
        Self {
            id,
            block_size,
            asset,
            buffer
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
