mod app;
mod config;
mod pane;
mod theme;
mod tree;
mod window;

use gtk4::glib;
use gtk4::prelude::*;

fn main() -> glib::ExitCode {
    gtk4::init().expect("failed to initialize GTK");
    libadwaita::init().expect("failed to initialize libadwaita");

    let app = app::build();
    app.run()
}
