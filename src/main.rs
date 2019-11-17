extern crate cpal;
extern crate sample;
#[macro_use]
extern crate log;
extern crate claxon;
extern crate raw_window_handle;
extern crate tokio;
extern crate vst;
extern crate winit;
#[macro_use]
extern crate failure;
extern crate generational_arena;

mod asset;
mod clock;
mod device;
mod linker;
mod plugin;

use self::asset::AssetSampleOutput;
use self::device::SysOutputDevice;
use self::linker::{Linker, PipeIndex, SampleDevice, SampleInput, SampleOutput};
use self::plugin::VstPlugin;

use cpal::traits::{DeviceTrait, HostTrait};
use generational_arena::{Arena, Index};
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use vst::api::{self, TimeInfo};
use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::host::{Host, HostBuffer, PluginInstance, PluginLoader};
use vst::plugin::{Info, Plugin};

type PluginIndex = Index;

use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct VSTHost {
    time_info: Option<TimeInfo>,
    block_size: isize,
}

impl VSTHost {
    pub fn new(block_size: isize) -> Self {
        Self {
            time_info: None,
            block_size,
        }
    }
}

impl Host for VSTHost {
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

pub struct Supervisor {
    linker: Linker,
    cpal_host: cpal::Host,
    cpal_loop: cpal::EventLoop,
    main_output: SysOutputDevice,
    vst_host: Arc<Mutex<VSTHost>>,
    pub plugins: BTreeMap<u64, VstPlugin>,
}

impl Supervisor {
    pub fn new() -> Self {
        let cpal_host = cpal::default_host();
        let cpal_loop = cpal_host.event_loop();
        let main_output =
            SysOutputDevice::new(cpal_host.default_output_device().expect("Output device"));
        info!(
            "Sample rate: {}, block size: {}",
            main_output.get_sample_rate(),
            main_output.get_block_size()
        );
        Self {
            linker: Linker::new(),
            vst_host: Arc::new(Mutex::new(VSTHost::new(
                main_output.get_block_size() as isize
            ))),
            cpal_host,
            cpal_loop,
            main_output,
            plugins: BTreeMap::new(),
        }
    }

    pub fn load_vst<T: AsRef<Path>>(&mut self, path: T) -> u64 {
        let mut loader = PluginLoader::load(path.as_ref(), self.vst_host.clone()).unwrap();
        let mut instance = loader.instance().unwrap();
        let mut plugin = VstPlugin::init(
            instance,
            self.main_output.get_sample_rate() as f32,
            self.main_output.get_block_size() as i64,
            &mut self.linker,
        );
        // plugin.load_editor(win_handle);
        let id = plugin.id;
        self.plugins.insert(plugin.id, plugin);
        id
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "trace");
    std::env::set_var("RUST_BACKTRACE", "full");
    let log = env_logger::init();

    let media = asset::AudioAsset::from_flac_file(
        std::fs::OpenOptions::new()
            .read(true)
            .open("example/sample.flac")
            .unwrap(),
    )
    .expect("Sample");
    let mut supervisor = Supervisor::new();
    let bsize = supervisor.main_output.block_size();
    let media_output = supervisor
        .linker
        .register_output(Box::new(AssetSampleOutput::new(media, bsize)));
    let log_input = supervisor
        .linker
        .register_input(Box::new(LoggerSample::new(bsize)));

    let plug = supervisor.load_vst(Path::new(
        "lib/vst-rs/target/debug/examples/gain_effect.dll",
    ));
    let plug2 = supervisor.load_vst(Path::new(
        "lib/vst-rs/target/debug/examples/gain_effect.dll",
    ));
    let entry = supervisor
        .linker
        .pipe(media_output, supervisor.plugins[&plug].get_inputs())
        .expect("Pipe flac -> vst");
    let last = supervisor
        .linker
        .pipe(
            supervisor.plugins[&plug].get_outputs(),
            supervisor.plugins[&plug2].get_inputs(),
        )
        .expect("Pipe vst -> vst");
    let last = supervisor
        .linker
        .pipe(supervisor.plugins[&plug2].get_outputs(), log_input)
        .expect("Pipe vst -> logger");

    let mut tmp_left = vec![0f32; bsize];
    let mut tmp_right = vec![0f32; bsize];
    let linker = &mut supervisor.linker;
    let plugins = &mut supervisor.plugins;

    let mut actual = entry;
    loop {
        linker
            .bind(
                actual,
                &mut [&mut tmp_left, &mut tmp_right],
                |mut audio_buffer, vst| {
                    if let Some(vst) = vst {
                        plugins.get_mut(&vst).unwrap().next(&mut audio_buffer);
                    }
                },
            )
            .expect("Wrong pipe");
        if let Some(next) = linker.get_next(actual) {
            actual = next;
        } else {
            info!("All sequence done (last: {:?}) !", actual);
            break;
        }
    }
    info!("{:?}", &tmp_left[0..4]);
    // while offset + bsize < media.buffer.len() {
    // for plugin in supervisor.plugins.iter_mut() {
    // let chunk = &media.buffer[offset..offset + bsize];
    // plugin.next(
    // &[&chunk, &chunk],
    // &mut [&mut output_left, &mut output_right],
    // );
    // info!("{:?} {:?}", &chunk[0..4], &output_left[0..4]);
    // }
    // offset += bsize;
    // }
    Ok(())
}

use std::borrow::Borrow;
use std::cell::RefCell;
pub struct LoggerSample {
    id: u64,
    buffer: RefCell<Vec<f32>>,
}

impl LoggerSample {
    fn new(size: usize) -> Self {
        let id = crate::linker::new_id();
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
