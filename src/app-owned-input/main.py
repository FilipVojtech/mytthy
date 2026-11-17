#!/usr/bin/python3
"""GTK owns the draft. A shell adapter authorizes and accepts submissions."""

import os
from pathlib import Path
import secrets
import sys
from tempfile import TemporaryDirectory

os.environ["GDK_BACKEND"] = "wayland"

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
gi.require_version("Vte", "3.91")
from gi.repository import Gdk, Gio, GLib, Gtk, Vte

PROMPT = "vte.ext.mytthy.prompt"


class ZshAdapter:
    def __init__(self):
        self.token = secrets.token_hex(16)
        self.generation = None
        self.last_sent = 0
        self.pid = None
        self.writer = None
        self.state = TemporaryDirectory(prefix="mytthy-zsh-")

    def register_properties(self):
        Vte.install_termprop(PROMPT, Vte.PropertyType.STRING, Vte.PropertyFlags(0))

    def start(self, terminal, on_state_changed, on_spawn_error):
        def prompt_changed(terminal, _name):
            marker, _ = terminal.get_termprop_string(PROMPT)
            self.observe_prompt(marker)
            # VTE limits what its termprop callbacks may call on the widget.
            GLib.idle_add(on_state_changed)

        def spawned(_terminal, pid, error, _data):
            if error is not None:
                on_spawn_error(error)
            else:
                self.pid = pid
                on_state_changed()

        terminal.connect("termprop-changed::" + PROMPT, prompt_changed)
        reader, self.writer = os.pipe()
        os.set_blocking(self.writer, False)
        self.record_limit = os.fpathconf(self.writer, "PC_PIPE_BUF")
        env = dict(
            os.environ,
            TERM="xterm-256color",
            ZDOTDIR=str(Path(__file__).resolve().parent / "adapter"),
            MYT_ORIGINAL_ZDOTDIR=os.environ.get("ZDOTDIR", ""),
            MYT_TOKEN=self.token,
            MYT_HISTORY=str(Path(self.state.name) / "history"),
        )
        terminal.spawn_with_fds_async(
            Vte.PtyFlags.DEFAULT, None, ["/usr/bin/zsh", "-d", "-i"],
            [f"{key}={value}" for key, value in env.items()], [reader], [3],
            GLib.SpawnFlags.DEFAULT, None, None, -1, None, spawned, None,
        )

    def observe_prompt(self, marker):
        self.generation = None
        if marker and marker.startswith(self.token + ":"):
            generation = marker.split(":", 1)[1]
            if generation.isascii() and generation.isdecimal() and len(generation) <= 15:
                self.generation = int(generation)

    def is_idle(self, terminal):
        if self.generation is None or self.generation <= self.last_sent or self.pid is None:
            return False
        pty = terminal.get_pty()
        try:
            return pty is not None and os.tcgetpgrp(pty.get_fd()) == self.pid
        except OSError:
            return False

    def submit(self, terminal, draft):
        if "\0" in draft:
            raise ValueError("Command contains a NUL character")
        encoded = "".join(f"\\x{byte:02x}" for byte in draft.encode("utf-8"))
        record = f"{self.generation} {encoded}\n".encode("ascii")
        # A single write of at most PIPE_BUF bytes either queues a complete
        # request or fails. The GTK thread never waits on Zsh.
        if len(record) > self.record_limit:
            raise ValueError("Command exceeds the prototype's submission limit")
        os.write(self.writer, record)
        self.last_sent = self.generation
        terminal.feed_child(list(f"\x1b[mytthy-{self.token}~".encode("ascii")))

    def close(self):
        if self.writer is not None:
            os.close(self.writer)
        self.state.cleanup()


class Prototype(Gtk.Application):
    def __init__(self, adapter):
        super().__init__(
            application_id="io.mytthy.AppOwnedInput", flags=Gio.ApplicationFlags.NON_UNIQUE
        )
        self.adapter = adapter
        self.exit_code = 0

    def do_activate(self):
        self.adapter.register_properties()
        self.window = Gtk.ApplicationWindow(application=self, title="MYT-006 Command Editor")
        self.window.set_default_size(640, 420)

        layout = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        self.window.set_child(layout)
        self.terminal = Vte.Terminal(vexpand=True)
        self.terminal.connect("child-exited", lambda *_: self.quit())
        layout.append(self.terminal)

        self.status = Gtk.Label(label="PTY input", xalign=0)
        layout.append(self.status)
        self.editor = Gtk.TextView(monospace=True, wrap_mode=Gtk.WrapMode.CHAR)
        self.editor.set_sensitive(False)
        scroll = Gtk.ScrolledWindow(min_content_height=96, max_content_height=96)
        scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroll.set_child(self.editor)
        layout.append(scroll)

        keys = Gtk.EventControllerKey()
        keys.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
        keys.connect("key-pressed", self.on_key)
        self.window.add_controller(keys)
        self.window.present()
        self.terminal.grab_focus()
        try:
            self.adapter.start(self.terminal, self.sync_prompt, self.on_spawn_error)
        except OSError as error:
            self.on_spawn_error(error)

    def do_shutdown(self):
        self.adapter.close()
        Gtk.Application.do_shutdown(self)

    def on_spawn_error(self, error):
        print(f"Could not launch shell in VTE: {error}", file=sys.stderr)
        self.exit_code = 1
        self.quit()

    def sync_prompt(self):
        idle = self.adapter.is_idle(self.terminal)
        if self.editor.get_sensitive() != idle:
            self.terminal.set_focusable(not idle)
            self.editor.set_sensitive(idle)
            (self.editor if idle else self.terminal).grab_focus()
        self.status.set_label("Enter submits. Shift+Enter adds a line." if idle else "PTY input")

    def on_key(self, _controller, keyval, _keycode, modifiers):
        mods = modifiers & Gtk.accelerator_get_default_mod_mask()
        if mods == Gdk.ModifierType.CONTROL_MASK and keyval in (Gdk.KEY_c, Gdk.KEY_C):
            self.interrupt_or_copy()
            return True
        if not self.adapter.is_idle(self.terminal):
            return False
        if keyval not in (Gdk.KEY_Return, Gdk.KEY_KP_Enter):
            return False
        if mods == Gdk.ModifierType.SHIFT_MASK:
            buffer = self.editor.get_buffer()
            buffer.delete_selection(True, True)
            buffer.insert_at_cursor("\n")
        elif not mods:
            self.submit()
        else:
            return False
        return True

    def submit(self):
        buffer = self.editor.get_buffer()
        try:
            self.adapter.submit(self.terminal, buffer.get_text(*buffer.get_bounds(), True))
        except (OSError, ValueError) as error:
            self.status.set_label(f"Could not submit command · {error}")
            return
        buffer.set_text("")
        self.sync_prompt()

    def interrupt_or_copy(self):
        if self.adapter.is_idle(self.terminal):
            buffer = self.editor.get_buffer()
            if buffer.get_has_selection():
                buffer.copy_clipboard(self.editor.get_clipboard())
                return
            if self.terminal.get_has_selection():
                self.terminal.copy_clipboard_format(Vte.Format.TEXT)
                return
            buffer.set_text("")
        self.terminal.feed_child([3])


if __name__ == "__main__":
    app = Prototype(ZshAdapter())
    status = app.run(sys.argv)
    sys.exit(app.exit_code or status)
