#!/usr/bin/python3
"""GTK4 + VTE / ZLE feasibility prototype. See the repository README."""

import os
from pathlib import Path
import sys

# Set before GTK initializes so this window cannot fall back to XWayland.
os.environ["GDK_BACKEND"] = "wayland"

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
gi.require_version("Vte", "3.91")
from gi.repository import Gdk, Gio, GLib, Gtk, Vte

PROPERTY = "vte.ext.mytthy.zle"


class Prototype(Gtk.Application):
    def __init__(self):
        super().__init__(
            application_id="io.mytthy.ClickPrototype", flags=Gio.ApplicationFlags.NON_UNIQUE
        )
        self.exit_code = 0
        self.press = None
        self.trace = os.environ.get("MYT_TRACE") == "1"
        self.writer = None

    def do_activate(self):
        Vte.install_termprop(PROPERTY, Vte.PropertyType.STRING, Vte.PropertyFlags(0))
        self.window = Gtk.ApplicationWindow(application=self, title="MYT-006 Click Prototype")
        self.window.set_default_size(640, 340)
        self.terminal = Vte.Terminal()
        self.terminal.set_enable_bidi(False)
        self.terminal.set_enable_shaping(False)
        self.terminal.set_scroll_on_output(True)
        css = Gtk.CssProvider()
        css.load_from_data(b"vte-terminal { padding: 0; }")
        Gtk.StyleContext.add_provider_for_display(
            self.window.get_display(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        self.window.set_child(self.terminal)

        # Observe events in capture without claiming them; VTE still owns drag selection.
        events = Gtk.EventControllerLegacy()
        events.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
        events.connect("event", self.on_pointer)
        self.window.add_controller(events)
        self.terminal.connect("child-exited", lambda *_: self.quit())
        if self.trace:
            self.terminal.connect("termprop-changed::" + PROPERTY, self.log_input)
            self.terminal.connect("selection-changed", self.log_selection)

        self.window.present()
        self.terminal.grab_focus()
        reader, self.writer = os.pipe()
        os.set_blocking(self.writer, False)
        env = dict(os.environ, TERM="xterm-256color", ZDOTDIR=str(Path(__file__).resolve().parent))
        envv = [f"{key}={value}" for key, value in env.items()]
        # VTE takes ownership of reader and maps it to FD 3 in the child.
        self.terminal.spawn_with_fds_async(
            Vte.PtyFlags.DEFAULT, None, ["/usr/bin/zsh", "-d", "-i"], envv,
            [reader], [3], GLib.SpawnFlags.DEFAULT, None, None, -1, None, self.on_spawn, None,
        )

    def do_shutdown(self):
        if self.writer is not None:
            os.close(self.writer)
        Gtk.Application.do_shutdown(self)

    def on_spawn(self, terminal, pid, error, data):
        if error is not None:
            print(f"Could not launch Zsh in VTE: {error.message}", file=sys.stderr)
            self.exit_code = 1
            self.quit()

    def log_input(self, terminal, name):
        print("ZLE", terminal.get_termprop_string(PROPERTY), flush=True)

    def log_selection(self, terminal):
        print("selection", terminal.get_has_selection(), flush=True)

    def on_pointer(self, controller, _event):
        # PyGObject does not marshal the event signal argument on GTK 4.22.
        event = controller.get_current_event()
        if event is None:
            return False
        kind = event.get_event_type()
        if kind not in (Gdk.EventType.BUTTON_PRESS, Gdk.EventType.BUTTON_RELEASE,
                        Gdk.EventType.MOTION_NOTIFY):
            return False
        ok, x, y = event.get_position()
        if not ok:
            self.press = None
            return False
        offset_x, offset_y = self.window.get_surface_transform()
        x, y = self.window.translate_coordinates(self.terminal, x + offset_x, y + offset_y)

        if kind == Gdk.EventType.MOTION_NOTIFY:
            if self.press is not None and self.dragged(self.press, x, y):
                self.press = None
        elif event.get_button() == Gdk.BUTTON_PRIMARY:
            if kind == Gdk.EventType.BUTTON_PRESS:
                self.press = None
                if (0 <= x < self.terminal.get_width() and 0 <= y < self.terminal.get_height()
                        and not event.get_modifier_state() & Gtk.accelerator_get_default_mod_mask()):
                    snapshot, _ = self.terminal.get_termprop_string(PROPERTY)
                    self.press = (x, y, snapshot)
            else:
                press, self.press = self.press, None
                if press and press[2] is not None and not self.dragged(press, x, y):
                    GLib.idle_add(self.place_caret, x, y, press[2])
        return False

    def dragged(self, press, x, y):
        return self.terminal.drag_check_threshold(*map(int, (*press[:2], x, y)))

    def place_caret(self, x, y, snapshot):
        terminal = self.terminal
        if terminal.get_has_selection() or terminal.get_termprop_string(PROPERTY)[0] != snapshot:
            return
        try:
            generation, offset, length, columns = map(int, snapshot.split())
        except ValueError:
            return
        if columns != terminal.get_column_count() or not 0 <= offset <= length:
            return

        # At the bottom, the adjustment gives the absolute row of the top cell.
        # For the fixture, each buffer character occupies one consecutive cell.
        scroll = terminal.get_vadjustment()
        top = scroll.get_value()
        if top != scroll.get_upper() - scroll.get_page_size():
            return
        column = int(x // terminal.get_char_width())
        row = int(y // terminal.get_char_height())
        if not (0 <= column < columns and 0 <= row < terminal.get_row_count()):
            return
        caret_col, caret_row = terminal.get_cursor_position()
        target = (int(top) + row - caret_row) * columns + column - caret_col + offset
        if 0 <= target <= length:
            try:
                os.write(self.writer, f"{generation} {target}\n".encode("ascii"))
            except OSError as error:
                print(f"Could not send caret position to ZLE: {error}", file=sys.stderr)
            if self.trace:
                print("click", column, int(top) + row, "target", target, flush=True)


if __name__ == "__main__":
    app = Prototype()
    status = app.run(sys.argv)
    sys.exit(app.exit_code or status)
