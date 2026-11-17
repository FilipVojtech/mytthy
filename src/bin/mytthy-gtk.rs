use mytthy::session;
use mytthy::settings::Settings;
use std::cell::Cell;
use vte4::gtk::{self, gdk, gio, glib};
use vte4::prelude::*;

const SHELL_STATE_PROPERTY: &std::ffi::CStr = c"vte.ext.mytthy.shell-state";

fn main() -> glib::ExitCode {
    gdk::set_allowed_backends("wayland");
    // SAFETY: The property name is a static C string. This runs once, before VTE creates any terminal.
    unsafe {
        vte4::ffi::vte_install_termprop(
            SHELL_STATE_PROPERTY.as_ptr(),
            vte4::ffi::VTE_PROPERTY_STRING,
            0,
        );
    }
    let app = gtk::Application::builder()
        .application_id("io.github.mytthy")
        .build();
    app.connect_activate(open_terminal);
    app.run()
}

fn open_terminal(app: &gtk::Application) {
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }

    let terminal = vte4::Terminal::new();
    let shell_state = Cell::new(session::ShellState::default());
    let property_name = SHELL_STATE_PROPERTY.to_str().expect("ASCII termprop name");
    terminal.connect_termprop_changed(Some(property_name), move |terminal, name| {
        let (marker, _) = terminal.termprop_string(name);
        let state = session::ShellState::from_marker(marker.as_deref());
        if shell_state.replace(state) != state {
            glib::g_debug!("mytthy", "Shell state: {state:?}");
        }
    });

    let settings = Settings::default();
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("mytthy")
        .default_width(settings.window_width)
        .default_height(settings.window_height)
        .child(&terminal)
        .build();

    let window_on_exit = window.downgrade();
    terminal.connect_child_exited(move |_, _| {
        if let Some(window) = window_on_exit.upgrade() {
            window.close();
        }
    });

    let spawn_cancel = gio::Cancellable::new();
    let cancel_on_close = spawn_cancel.clone();
    window.connect_close_request(move |_| {
        cancel_on_close.cancel();
        glib::Propagation::Proceed
    });

    window.present();
    terminal.grab_focus();

    let shell = session::shell_executable();
    let launch_error = format!("Could not start {shell}");
    let window_on_spawn = window.downgrade();
    // Keep the loop alive to finish cancellation if the window closes during spawn.
    let spawn_hold = app.hold();
    terminal.spawn_async(
        vte4::PtyFlags::DEFAULT,
        None,
        &[&shell],
        &[],
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        Some(&spawn_cancel),
        move |result| {
            let _hold = spawn_hold;
            let Err(error) = result else { return };
            if error.matches(gio::IOErrorEnum::Cancelled) {
                return;
            }

            let message = format!("{launch_error}\n{error}");
            eprintln!("{message}");
            if let Some(window) = window_on_spawn.upgrade() {
                let label = gtk::Label::new(Some(&message));
                label.set_wrap(true);
                label.set_selectable(true);
                window.set_child(Some(&label));
            }
        },
    );
}
