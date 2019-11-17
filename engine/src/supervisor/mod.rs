use crate::{devices::*, prelude::*};
use cpal::traits::HostTrait;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex, RwLock},
};
use vst::host::PluginLoader;
pub mod linker;

pub struct Supervisor {
    pub linker: Linker,
    pub cpal_host: cpal::Host,
    pub cpal_loop: cpal::EventLoop,
    pub main_output: SysOutputDevice,
    pub vst_host: Arc<Mutex<VstHost>>,
    pub plugins: BTreeMap<VstId, VstPlugin>,
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
            vst_host: Arc::new(Mutex::new(VstHost::new(
                main_output.get_block_size() as isize
            ))),
            cpal_host,
            cpal_loop,
            main_output,
            plugins: BTreeMap::new(),
        }
    }

    pub fn load_vst<T: AsRef<Path>>(&mut self, path: T) -> VstId {
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
