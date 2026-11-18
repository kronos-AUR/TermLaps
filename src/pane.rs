//! A single terminal pane: a VTE widget running a command on a PTY,
//! with the "smart Ctrl+C" behavior:
//!
//! * `Ctrl+C` with an active mouse selection  -> copies the selection
//! * `Ctrl+C` with no selection                -> passed through to the
//!   child process as `0x03`, which the line discipline turns into
//!   `SIGINT` (kills the foreground process)
//! * `Ctrl+V` pastes the clipboard (in addition to the classic
//!   `Ctrl+Shift+C` / `Ctrl+Shift+V` handled by VTE itself).

use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::{gdk, gio, glib, pango};
use vte4::prelude::*;
use vte4::Terminal;

use crate::config::{Profile, Settings};

pub struct TerminalPane {
    pub id: u64,
    pub profile: Profile,
    /// Outer widget of the pane (what gets placed inside tabs / paned splits).
    pub container: gtk::Box,
    pub term: Terminal,
}

impl TerminalPane {
    pub fn new(
        id: u64,
        profile: Profile,
        settings: &Settings,
        on_exit: Rc<dyn Fn(u64)>,
    ) -> Rc<RefCell<Self>> {
        let term = Terminal::new();

        // --- Theme -----------------------------------------------------------
        let palette = profile.theme.palette();
        let color_refs: Vec<&gdk::RGBA> = palette.colors.iter().collect();
        term.set_colors(
            Some(&palette.foreground),
            Some(&palette.background),
            &color_refs,
        );

        // --- Font / scrollback -------------------------------------------------
        let font = profile
            .font
            .clone()
            .or_else(|| settings.font.clone())
            .unwrap_or_else(|| "Monospace 12".to_string());
        term.set_font(Some(&pango::FontDescription::from_string(&font)));
        if let Some(lines) = settings.scrollback_lines {
            term.set_scrollback_lines(lines);
        }
        term.set_mouse_autohide(true);

        // --- Spawn the command on a new PTY ------------------------------------
        let argv: Vec<&str> = profile.command.iter().map(String::as_str).collect();
        let envv: Vec<&str> = profile.env.iter().map(String::as_str).collect();
        let working_directory = profile.working_directory.clone();
        let spawn_cmd = profile.command.clone();
        term.spawn_async(
            vte4::PtyFlags::DEFAULT,
            working_directory.as_deref(),
            &argv,
            &envv,
            glib::SpawnFlags::DEFAULT,
            || {},
            -1,
            None::<&gio::Cancellable>,
            move |result| {
                if let Err(err) = result {
                    eprintln!("termlaps: failed to spawn {spawn_cmd:?}: {err}");
                }
            },
        );

        // --- Notify the window when the child exits ----------------------------
        {
            let cb = on_exit.clone();
            term.connect_child_exited(move |_, _status| cb(id));
        }

        // --- Smart Ctrl+C / Ctrl+V ---------------------------------------------
        // Capture phase so we run before VTE's own key handling. Returning
        // `Proceed` lets VTE deliver the key to the child (Ctrl+C -> 0x03 ->
        // SIGINT); returning `Stop` swallows the key after we handled it.
        let key_controller = gtk::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        {
            let term = term.clone();
            key_controller.connect_key_pressed(move |_, keyval, _keycode, state| {
                let ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
                let shift = state.contains(gdk::ModifierType::SHIFT_MASK);
                if !ctrl || shift {
                    // Leave Ctrl+Shift+C / Ctrl+Shift+V to VTE's defaults.
                    return glib::Propagation::Proceed;
                }
                match keyval {
                    gdk::Key::c | gdk::Key::C => {
                        if term.has_selection() {
                            term.copy_clipboard_format(vte4::Format::Text);
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    gdk::Key::v | gdk::Key::V => {
                        term.paste_clipboard();
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            });
        }
        term.add_controller(key_controller);

        // --- Widget ------------------------------------------------------------
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_child(Some(&term));
        scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.append(&scrolled);
        container.set_hexpand(true);
        container.set_vexpand(true);

        Rc::new(RefCell::new(TerminalPane {
            id,
            profile,
            container,
            term,
        }))
    }
}
