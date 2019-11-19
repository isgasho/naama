extern crate claxon;
pub extern crate cpal;
pub extern crate sample;
pub extern crate vst;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate generational_arena;

pub mod devices;
pub mod loader;
pub mod prelude;
pub mod supervisor;

#[cfg(test)]
mod tests {
    #[test]
    fn all() {
        std::env::set_var("RUST_LOG", "trace");
        std::env::set_var("RUST_BACKTRACE", "full");
        env_logger::init();
        let media = AudioAsset::from_flac_file(
            std::fs::OpenOptions::new()
                .read(true)
                .open("examples/assets/sample.flac")
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
        let plug = supervisor.load_vst(Path::new("examples/vst/gain_effect.dll"));
        let plug2 = supervisor.load_vst(Path::new("examples/vst/gain_effect.dll"));
        let entry = supervisor
            .linker
            .pipe(media_output, supervisor.plugins[&plug].get_inputs())
            .expect("Pipe flac -> vst");
        supervisor
            .linker
            .pipe(
                supervisor.plugins[&plug].get_outputs(),
                supervisor.plugins[&plug2].get_inputs(),
            )
            .expect("Pipe vst -> vst");
        supervisor
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
    }
}
