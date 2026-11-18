//! Application setup: configuration, keyboard shortcuts, main window.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gio;
use gtk4::prelude::*;
use libadwaita as adw;

use crate::config;
use crate::window::AppWindow;

pub fn build() -> adw::Application {
    let app = adw::Application::new(
        Some("com.github.termlaps.Terminal"),
        gio::ApplicationFlags::empty(),
    );

    let config = Rc::new(config::load());
    let window: Rc<RefCell<Option<Rc<AppWindow>>>> = Rc::new(RefCell::new(None));

    app.connect_startup(move |app| {
        let app = app.clone();

        app.set_accels_for_action("win.new-tab", &["<Ctrl><Shift>t"]);
        app.set_accels_for_action("win.split", &["<Ctrl><Shift>s"]);
        app.set_accels_for_action("win.close-pane", &["<Ctrl><Shift>w"]);
        app.set_accels_for_action("win.next-tab", &["<Ctrl>Page_Down"]);
        app.set_accels_for_action("win.prev-tab", &["<Ctrl>Page_Up"]);
        app.set_accels_for_action("app.quit", &["<Ctrl>q"]);

        let quit = gio::SimpleAction::new("quit", None);
        let quit_app = app.clone();
        quit.connect_activate(move |_, _| quit_app.quit());
        app.add_action(&quit);
    });

    app.connect_activate(move |app| {
        let mut slot = window.borrow_mut();
        if slot.is_none() {
            *slot = Some(AppWindow::new(app, config.clone()));
        }
        if let Some(w) = slot.as_ref() {
            w.window.present();
        }
    });

    app
}
