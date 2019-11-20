extern crate engine;
#[macro_use]
extern crate clap;

use clap::{App, SubCommand, Arg};
use engine::prelude::*;
use engine::supervisor::Supervisor;
use std::path::{Path, PathBuf};

fn main() {
    let matches = App::new("naama-cli")
        .version("1.0")
        .author("Asya c. <asya.corbeau.dev@gmail.com>")
        .about("A simple VST host")
        .arg(Arg::with_name("vst").short("v").long("vst").required(true).takes_value(true).multiple(true).help("Load a VST from its path"))
        .arg(Arg::with_name("sample").short("s").required(true).long("sample").takes_value(true).multiple(true).help("Load a `.flac` sample from its path"))
        .get_matches();
    for vst in matches.values_of("vst").unwrap() {

    }
    let supervisor = engine::supervisor::Supervisor::new();
    println!("Hello, world!");
}
