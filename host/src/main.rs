#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate clap;
extern crate engine;

mod gui;

use gtk::{Application, ApplicationWindow, Button};

fn main() {
    gtk::init().expect("Initialize GTK");
    let builder = gui::get_main_window();
    let window = builder
        .get_object("win_host")
        .expect("Wrong layout: invalide window name");
    window.show_all();
    gtk::main();
}
