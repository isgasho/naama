use crate::prelude::*;
use generational_arena::{Arena, Index};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use vst::buffer::AudioBuffer;
use vst::host::HostBuffer;

#[derive(Debug, Fail)]
pub enum LinkerError {
    #[fail(display = "Invalide input index: {:?}", 0)]
    InvalideInput(InputIndex),
    #[fail(display = "Invalide output index: {:?}", 0)]
    InvalideOutput(OutputIndex),
    #[fail(display = "Invalide pipe index: {:?}", 0)]
    InvalidePipe(PipeIndex),
    #[fail(display = "Both input and output buffer size must be the same for piping")]
    PipeBufferMalformated,
    #[fail(display = "Pipe must get the same number of inputs and outputs")]
    PipeWrongIO,
}

/// Index of an allocated input device int the linker arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct InputIndex(Index);
impl From<Index> for InputIndex {
    fn from(idx: Index) -> Self {
        Self(idx)
    }
}

/// Index of an allocated output device int the linker arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct OutputIndex(Index);
impl From<Index> for OutputIndex {
    fn from(idx: Index) -> Self {
        Self(idx)
    }
}

/// Index of an allocated pipe of two device int the linker arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct PipeIndex(Index);
impl From<Index> for PipeIndex {
    fn from(idx: Index) -> Self {
        Self(idx)
    }
}

/// Unique identifier of an I/O device
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeviceId(pub u64);

static mut last_id: Option<Arc<RwLock<u64>>> = None;

/// Generate a new unique identifier
pub fn new_id() -> u64 {
    unsafe {
        if last_id.is_none() {
            last_id = Some(Arc::new(RwLock::new(0)));
        }
        let mut id = last_id.as_mut().unwrap().write().unwrap();
        *id += 1;
        *id
    }
}

pub trait SampleDevice {
    /// Get the sample block size
    fn block_size(&self) -> usize;

    /// Get channels count
    fn nbr_channel(&self) -> usize;

    /// Get uniq identifier
    fn id(&self) -> DeviceId;

    /// If the device was loaded from a VST parent_vst must return his instance id
    /// otherwise the `process` methode of the vst plugin will not be called
    fn parent_vst(&self) -> Option<VstId> {
        None
    }
}

pub trait SampleInput: SampleDevice {
    fn next(&mut self, buffer: &[f32], channel: usize) {
        unimplemented!()
    }
}

pub trait SampleOutput: SampleDevice {
    fn next(&self, channel: usize) -> Option<std::sync::RwLockReadGuard<Vec<f32>>> {
        unimplemented!()
    }
}

pub struct SamplePipe {
    inputs: InputIndex,
    outputs: OutputIndex,
    buffer: HostBuffer<f32>,
}

pub struct Linker {
    output_devices: Arena<Box<dyn SampleOutput>>,
    input_devices: Arena<Box<dyn SampleInput>>,
    pipes: Arena<SamplePipe>,
    sequences: BTreeMap<PipeIndex, PipeIndex>,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            output_devices: Arena::new(),
            input_devices: Arena::new(),
            pipes: Arena::new(),
            sequences: BTreeMap::new(),
        }
    }

    pub fn get_next(&self, actual: PipeIndex) -> Option<PipeIndex> {
        self.sequences.get(&actual).map(|e| *e)
    }

    pub fn register_input(&mut self, input: Box<dyn SampleInput>) -> InputIndex {
        self.input_devices.insert(input).into()
    }

    pub fn register_output(&mut self, output: Box<dyn SampleOutput>) -> OutputIndex {
        self.output_devices.insert(output).into()
    }

    pub fn get_pipe<'a>(&'a mut self, idx: PipeIndex) -> Option<&'a mut SamplePipe> {
        self.pipes.get_mut(idx.0)
    }

    pub fn bind<'a, F: (FnMut(AudioBuffer<f32>, Option<VstId>))>(
        &'a mut self,
        idx: PipeIndex,
        tmp: &mut [&mut [f32]],
        mut process: F,
    ) -> Result<Option<PipeIndex>, LinkerError> {
        let inputs = &mut self.input_devices;
        let outputs = &mut self.output_devices;
        let pipe = self
            .pipes
            .get_mut(idx.0)
            .ok_or(LinkerError::InvalidePipe(idx))?;
        let out_ins = &outputs[pipe.outputs.0];
        let in_ins = &mut inputs[pipe.inputs.0];
        let vst = in_ins.parent_vst();
        let out_left = out_ins.next(0).expect("EOF");
        let out_right = out_ins.next(1).expect("EOF");
        let outputs = &[&out_left[0..out_left.len()], &out_right[0..out_right.len()]];
        let binding = pipe.buffer.bind(outputs, tmp);
        process(binding, vst);
        in_ins.next(tmp[0], 0);
        in_ins.next(tmp[1], 1);
        let next = self.sequences.get(&idx).map(|e| *e);
        Ok(next)
    }

    fn calc_sequences(&mut self) {
        self.sequences.clear();
        for (this, this_pipe) in self.pipes.iter() {
            let this_id = self.output_devices[this_pipe.inputs.0].id();
            for (other, other_pipe) in self.pipes.iter() {
                let other_id = self.input_devices[other_pipe.outputs.0].id();
                if this_id == other_id {
                    self.sequences.insert(this.into(), other.into());
                }
            }
        }
    }

    pub fn pipe(
        &mut self,
        output_idx: OutputIndex,
        input_idx: InputIndex,
    ) -> Result<PipeIndex, LinkerError> {
        let inputs = self
            .input_devices
            .get(input_idx.0)
            .ok_or(LinkerError::InvalideInput(input_idx))?;
        let outputs = self
            .output_devices
            .get(output_idx.0)
            .ok_or(LinkerError::InvalideOutput(output_idx))?;
        let buffer = HostBuffer::new(inputs.nbr_channel(), outputs.nbr_channel());
        let pipe = self.pipes.insert(SamplePipe {
            buffer,
            inputs: input_idx,
            outputs: output_idx,
        });
        self.calc_sequences();
        Ok(pipe.into())
    }
}
