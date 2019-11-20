#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate clap;
extern crate engine;
extern crate gtk;
extern crate gio;

use gtk::prelude::*;
use gio::prelude::*;

mod gui;

use gtk::{BuilderExtManual, Application, ApplicationWindow, Button, WidgetExt};

fn main() {
    let uiapp = gtk::Application::new(Some("org.gtkrsnotes.demo"),
                                      gio::ApplicationFlags::FLAGS_NONE)
                                 .expect("Application::new failed");
    uiapp.connect_activate(|app| {
        // We create the main window.
        let win = gtk::ApplicationWindow::new(app);

        // Then we set its size and a title.
        win.set_default_size(320, 200);
        win.set_title("Basic example");

        // Don't forget to make all widgets visible.
        win.show_all();
    });
    uiapp.run(&std::env::args().collect::<Vec<_>>());

}
