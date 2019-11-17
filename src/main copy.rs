extern crate cpal;
#[macro_use]
extern crate log;
extern crate raw_window_handle;
extern crate vst;
extern crate winit;
extern crate tokio;
mod clock;

use cpal::traits::{DeviceTrait, HostTrait};
use std::ffi::c_void;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use vst::api::{self, TimeInfo};
use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::host::{Host, HostBuffer, PluginInstance, PluginLoader};
use vst::plugin::Plugin;

use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct GuiHost {
    event_loop: EventLoop<()>,
    windows: Vec<Window>,
}

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

pub struct SysOutputDevice {
    device: cpal::Device,
    format: cpal::SupportedFormat,
}

impl SysOutputDevice {
    pub fn new(device: cpal::Device) -> Self {
        let mut formats_range = device
            .supported_output_formats()
            .expect("Wrong output device");
        let format = formats_range
            .next()
            .expect("Output device supported format");
        Self { device, format }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.format.max_sample_rate.0
    }

    pub fn get_block_size(&self) -> u32 {
        self.get_sample_rate() / 100 * (self.format.channels as u32)
    }
}

pub struct VstPlugin {
    buffer: HostBuffer<f32>,
    input: usize,
    output: usize,
    instance: PluginInstance,
}

impl VstPlugin {
    pub fn init(mut instance: PluginInstance, sample_rate: f32, block_size: i64) -> Self {
        let info = instance.get_info();
        instance.init();
        instance.set_sample_rate(sample_rate);
        instance.set_block_size(block_size);
        instance.resume();
        info!("Plugin initialized: {:?}", info);
        Self {
            input: info.inputs as usize,
            output: info.outputs as usize,
            instance,
            buffer: HostBuffer::new(info.inputs as usize, info.outputs as usize),
        }
    }

    pub fn load_editor(&mut self, win_handle: *mut c_void) {
        let edit = self.instance.get_editor().expect("Editor");
        info!("Editor size: W {}, H {}", edit.size().0, edit.size().1);
        self.instance.open_editor(win_handle);
    }

    pub fn next(&mut self) {
        let input = vec![vec![0f32; self.input]; self.input];
        let mut output = vec![vec![0f32; self.output]; self.output];
        self.instance
            .process(&mut self.buffer.bind(&input, &mut output));
    }
}

pub struct Supervisor {
    cpal_host: cpal::Host,
    cpal_loop: cpal::EventLoop,
    main_output: SysOutputDevice,
    vst_host: Arc<Mutex<VSTHost>>,
    plugins: Vec<VstPlugin>,
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
            vst_host: Arc::new(Mutex::new(VSTHost::new(
                main_output.get_block_size() as isize
            ))),
            cpal_host,
            cpal_loop,
            main_output,
            plugins: Vec::new(),
        }
    }

    pub fn load_vst<T: AsRef<Path>>(&mut self, path: T, win_handle: *mut c_void) {
        let mut loader = PluginLoader::load(path.as_ref(), self.vst_host.clone()).unwrap();
        let mut instance = loader.instance().unwrap();
        let mut plugin = VstPlugin::init(
            instance,
            self.main_output.get_sample_rate() as f32,
            self.main_output.get_block_size() as i64,
        );
        plugin.load_editor(win_handle);
        self.plugins.push(plugin);
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    let log = env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();
    if let raw_window_handle::RawWindowHandle::Windows(handle) =
        unsafe { window.raw_window_handle() }
    {
        let hnd = handle.hwnd as u64;
        let supervisor_handle = std::thread::spawn(move || {
            let mut supervisor = Supervisor::new();
            supervisor.load_vst(
                Path::new("C:/Program Files/Steinberg/VSTPlugins/VST Classics 1/Tunefish4.dll"),
                hnd as *mut c_void,
            );

            loop {
                for plug in supervisor.plugins.iter_mut() {
                    std::thread::sleep(std::time::Duration::from_nanos(1));
                    plug.next();
                }
            }
        });
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => *control_flow = ControlFlow::Wait,
        });
    } else {
        panic!("Unamaged window handle !");
    }
}
