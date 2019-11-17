use claxon::FlacReader;
use crate::prelude::*;
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
