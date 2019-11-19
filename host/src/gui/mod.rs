use gtk::Builder;

const main_window_src = include_str!("layouts/win_host.glade");

pub fn get_main_window() -> Builder {
    Builder::new_from_string(main_window_src)
}