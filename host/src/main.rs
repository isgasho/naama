#[macro_use]
extern crate log;
extern crate raw_window_handle;
extern crate winit;
#[macro_use]
extern crate failure;
extern crate engine;

use engine::devices::*;
use engine::loader::asset::*;
use engine::prelude::*;
use engine::supervisor::*;
use std::path::Path;

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    std::env::set_var("RUST_BACKTRACE", "full");
    env_logger::init();

    let media = AudioAsset::from_flac_file(
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
}
