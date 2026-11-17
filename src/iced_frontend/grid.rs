use super::session::{GridSize, Terminal};
use alacritty_terminal::{
    grid::{Dimensions, Scroll},
    index::{Column, Line, Point as CellPoint, Side},
    selection::{Selection, SelectionRange, SelectionType},
    term::{RenderableCursor, cell::Cell, cell::Flags, color::Colors},
    vte::ansi::{Color as AnsiColor, CursorShape, NamedColor, Rgb},
};
use iced::{
    Color, Event, Font, Length, Pixels, Point, Rectangle, Renderer, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse,
        renderer::{Quad, Renderer as _},
        text::{self, Paragraph, Renderer as _},
        widget::{Tree, tree},
    },
    alignment, font,
    keyboard::{self, Key},
};
use std::sync::Arc;

const PADDING: f32 = 8.0;

#[derive(Clone, Copy)]
pub(super) struct Metrics {
    font_size: f32,
    width: f32,
    height: f32,
}

impl Metrics {
    pub fn new(font_size: f32) -> Self {
        let height = (font_size * 1.25).ceil();
        let sample = text_style(font_size, height, Font::MONOSPACE, f32::INFINITY);
        let paragraph =
            <Renderer as text::Renderer>::Paragraph::with_text(sample.with_content("MMMMMMMM"));
        Self {
            font_size,
            width: (paragraph.min_width() / 8.0).max(1.0),
            height,
        }
    }

    pub fn grid_size(self, size: Size) -> GridSize {
        GridSize {
            columns: (((size.width - 2.0 * PADDING) / self.width) as u16).max(2),
            lines: (((size.height - 2.0 * PADDING) / self.height) as u16).max(1),
            cell_width: self.width.round() as u16,
            cell_height: self.height as u16,
        }
    }

    pub fn line_height(self) -> f32 {
        self.height
    }
}

pub(super) struct Grid<'a, Message> {
    pub metrics: Metrics,
    pub terminal: &'a Terminal,
    pub focused: bool,
    pub on_input: Box<dyn Fn(Vec<u8>) -> Message + 'a>,
}

impl<Message> Widget<Message, Theme, Renderer> for Grid<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PointerState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(PointerState {
            terminal: Arc::downgrade(self.terminal),
            ..PointerState::default()
        })
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<PointerState>();
        let identity = Arc::downgrade(self.terminal);
        if !state.terminal.ptr_eq(&identity) {
            *state = PointerState {
                terminal: identity,
                ..PointerState::default()
            };
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<PointerState>();
        let bounds = layout.bounds();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(bounds) =>
            {
                state.dragging = true;
                state.pointer = cursor.position();
                self.select_pointer(state, bounds, true);
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.dragging => {
                state.pointer = Some(*position);
                self.select_pointer(state, bounds, false);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.dragging => {
                self.select_pointer(state, bounds, false);
                state.dragging = false;
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if state.dragging => {
                let lines = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 3.0,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / self.metrics.height,
                };
                state.scroll_remainder += lines;
                let lines = state.scroll_remainder.trunc() as i32;
                state.scroll_remainder -= lines as f32;
                self.terminal.lock().scroll_display(Scroll::Delta(lines));
                self.select_pointer(state, bounds, false);
            }
            Event::Window(iced::window::Event::Unfocused) => {
                state.dragging = false;
                return;
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                if self.focused && modifiers.control() && modifiers.shift() =>
            {
                match key.as_ref() {
                    Key::Character(character) if character.eq_ignore_ascii_case("c") => {
                        if let Some(text) = self.terminal.lock().selection_to_string() {
                            clipboard.write(iced::advanced::clipboard::Kind::Standard, text);
                        }
                    }
                    Key::Character(character) if character.eq_ignore_ascii_case("v") => {
                        if let Some(text) =
                            clipboard.read(iced::advanced::clipboard::Kind::Standard)
                        {
                            let bracketed = self
                                .terminal
                                .lock()
                                .mode()
                                .contains(alacritty_terminal::term::TermMode::BRACKETED_PASTE);
                            shell.publish((self.on_input)(paste_bytes(&text, bracketed)));
                        }
                    }
                    _ => return,
                }
            }
            _ => return,
        }
        shell.capture_event();
        shell.request_redraw();
    }

    fn mouse_interaction(
        &self,
        _: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        _: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        // Release the terminal lock before shaping or drawing. Under load the
        // PTY worker can continue parsing while the renderer processes this frame.
        let (cells, columns, cursor, offset, colors, selection) = {
            use alacritty_terminal::grid::Dimensions;
            let terminal = self.terminal.lock();
            let content = terminal.renderable_content();
            (
                content
                    .display_iter
                    .map(|entry| entry.cell.clone())
                    .collect::<Vec<_>>(),
                terminal.columns(),
                content.cursor,
                content.display_offset,
                *content.colors,
                content.selection,
            )
        };

        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let background = palette_color(&colors, NamedColor::Background as usize);
        renderer.with_layer(clip, |renderer| {
            fill(renderer, bounds, background);
            for (line, cells) in cells.chunks(columns).enumerate() {
                self.draw_line(
                    renderer, bounds, cells, line, cursor, offset, &colors, background, selection,
                );
            }
        });
    }
}

impl<Message> Grid<'_, Message> {
    fn select_pointer(&self, state: &PointerState, bounds: Rectangle, start: bool) {
        let Some(pointer) = state.pointer else { return };
        let mut terminal = self.terminal.lock();
        let (point, side) = self.metrics.cell_at(
            pointer,
            bounds,
            terminal.columns(),
            terminal.screen_lines(),
            terminal.grid().display_offset(),
        );
        if start {
            terminal.selection = Some(Selection::new(SelectionType::Simple, point, side));
        } else if let Some(selection) = &mut terminal.selection {
            selection.update(point, side);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_line(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        cells: &[Cell],
        line: usize,
        cursor: RenderableCursor,
        offset: usize,
        colors: &Colors,
        background: Color,
        selection: Option<SelectionRange>,
    ) {
        let y = bounds.y + PADDING + line as f32 * self.metrics.height;
        let x = bounds.x + PADDING;
        let on_cursor_line = cursor.point.line.0 + offset as i32 == line as i32;
        let mut column = 0;

        while column < cells.len() {
            let cell = &cells[column];
            let on_cursor = on_cursor_line && cursor.point.column.0 == column;
            let cursor_shape = if on_cursor && !self.focused && cursor.shape != CursorShape::Hidden
            {
                CursorShape::HollowBlock
            } else if on_cursor {
                cursor.shape
            } else {
                CursorShape::Hidden
            };
            let selected = is_selected(selection, cell, line as i32 - offset as i32, column);
            let style = CellStyle::new(cell, colors, cursor_shape, selected);
            let mut width = if cell.flags.contains(Flags::WIDE_CHAR) {
                2
            } else {
                1
            };
            let mut content = String::new();
            if !cell
                .flags
                .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
            {
                content.push(if cell.flags.contains(Flags::HIDDEN) {
                    ' '
                } else {
                    cell.c
                });
                if !cell.flags.contains(Flags::HIDDEN) {
                    content.extend(cell.zerowidth().unwrap_or_default());
                }
            }

            // Batch monospace ASCII runs. Keep other characters at their grid
            // positions regardless of the advance chosen by fallback fonts.
            if cursor_shape == CursorShape::Hidden && plain_ascii(cell) {
                for (next_column, next) in cells.iter().enumerate().skip(column + 1) {
                    if (on_cursor_line && cursor.point.column.0 == next_column)
                        || !plain_ascii(next)
                        || CellStyle::new(
                            next,
                            colors,
                            CursorShape::Hidden,
                            is_selected(selection, next, line as i32 - offset as i32, next_column),
                        ) != style
                    {
                        break;
                    }
                    content.push(next.c);
                    width += 1;
                }
            }

            let cell_bounds = Rectangle::new(
                Point::new(x + column as f32 * self.metrics.width, y),
                Size::new(self.metrics.width * width as f32, self.metrics.height),
            );
            style.draw(renderer, cell_bounds, self.metrics, &content, background);
            column += width;
        }
    }
}

fn is_selected(selection: Option<SelectionRange>, cell: &Cell, line: i32, column: usize) -> bool {
    selection.is_some_and(|selection| {
        selection.contains(CellPoint::new(Line(line), Column(column)))
            || (cell.flags.contains(Flags::WIDE_CHAR)
                && selection.contains(CellPoint::new(Line(line), Column(column + 1))))
    })
}

fn plain_ascii(cell: &Cell) -> bool {
    cell.c.is_ascii()
        && !cell.flags.intersects(
            Flags::WIDE_CHAR
                | Flags::WIDE_CHAR_SPACER
                | Flags::LEADING_WIDE_CHAR_SPACER
                | Flags::HIDDEN,
        )
        && cell.zerowidth().is_none()
}

#[derive(Clone, Copy, PartialEq)]
struct CellStyle {
    foreground: Color,
    background: Color,
    cursor: CursorShape,
    cursor_color: Color,
    font: Font,
    underline: bool,
    strikeout: bool,
}

impl CellStyle {
    fn new(cell: &Cell, colors: &Colors, cursor: CursorShape, selected: bool) -> Self {
        let mut foreground = ansi_color(colors, cell.fg);
        let mut background = ansi_color(colors, cell.bg);
        if cell.flags.contains(Flags::DIM) {
            foreground = Color::from_rgb(
                foreground.r * 0.66,
                foreground.g * 0.66,
                foreground.b * 0.66,
            );
        }
        if cell.flags.contains(Flags::INVERSE) {
            std::mem::swap(&mut foreground, &mut background);
        }
        if selected {
            foreground = Color::WHITE;
            background = Color::from_rgb8(54, 93, 145);
        }
        let cursor_color = palette_color(colors, NamedColor::Cursor as usize);
        if cursor == CursorShape::Block {
            foreground = background;
            background = cursor_color;
        }
        Self {
            foreground,
            background,
            cursor,
            cursor_color,
            font: Font {
                weight: if cell.flags.contains(Flags::BOLD) {
                    font::Weight::Bold
                } else {
                    font::Weight::Normal
                },
                style: if cell.flags.contains(Flags::ITALIC) {
                    font::Style::Italic
                } else {
                    font::Style::Normal
                },
                ..Font::MONOSPACE
            },
            underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
            strikeout: cell.flags.contains(Flags::STRIKEOUT),
        }
    }

    fn draw(
        self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        metrics: Metrics,
        text: &str,
        background: Color,
    ) {
        if self.background != background {
            fill(renderer, bounds, self.background);
        }
        if !text.trim().is_empty() {
            let mut style = text_style(metrics.font_size, metrics.height, self.font, f32::INFINITY)
                .with_content(text.to_owned());
            if !text.is_ascii() {
                style.shaping = text::Shaping::Advanced;
            }
            renderer.fill_text(style, bounds.position(), self.foreground, bounds);
        }

        let horizontal = |renderer: &mut Renderer, y, height, color| {
            fill(
                renderer,
                Rectangle {
                    y,
                    height,
                    ..bounds
                },
                color,
            );
        };
        if self.underline {
            horizontal(
                renderer,
                bounds.y + bounds.height - 2.0,
                1.0,
                self.foreground,
            );
        }
        if self.strikeout {
            horizontal(
                renderer,
                bounds.y + bounds.height / 2.0,
                1.0,
                self.foreground,
            );
        }
        match self.cursor {
            CursorShape::Underline => horizontal(
                renderer,
                bounds.y + bounds.height - 2.0,
                2.0,
                self.cursor_color,
            ),
            CursorShape::Beam => fill(
                renderer,
                Rectangle {
                    width: 2.0,
                    ..bounds
                },
                self.cursor_color,
            ),
            CursorShape::HollowBlock => {
                renderer.fill_quad(
                    Quad {
                        bounds,
                        border: iced::Border {
                            width: 1.0,
                            color: self.cursor_color,
                            ..Default::default()
                        },
                        ..Quad::default()
                    },
                    Color::TRANSPARENT,
                );
            }
            CursorShape::Hidden | CursorShape::Block => {}
        }
    }
}

fn text_style(font_size: f32, height: f32, font: Font, width: f32) -> text::Text<()> {
    text::Text {
        content: (),
        bounds: Size::new(width, height),
        size: Pixels(font_size),
        line_height: text::LineHeight::Absolute(Pixels(height)),
        font,
        align_x: text::Alignment::Left,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Basic,
        wrapping: text::Wrapping::None,
    }
}

fn fill(renderer: &mut Renderer, bounds: Rectangle, color: Color) {
    renderer.fill_quad(
        Quad {
            bounds,
            ..Quad::default()
        },
        color,
    );
}

fn ansi_color(colors: &Colors, color: AnsiColor) -> Color {
    match color {
        AnsiColor::Spec(rgb) => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
        AnsiColor::Indexed(index) => palette_color(colors, index.into()),
        AnsiColor::Named(index) => palette_color(colors, index as usize),
    }
}

fn palette_color(colors: &Colors, index: usize) -> Color {
    let rgb = palette_rgb(colors, index);
    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
}

pub(super) fn palette_rgb(colors: &Colors, index: usize) -> Rgb {
    const ANSI: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 49, 49),
        (13, 188, 121),
        (229, 229, 16),
        (36, 114, 200),
        (188, 63, 188),
        (17, 168, 205),
        (229, 229, 229),
        (102, 102, 102),
        (241, 76, 76),
        (35, 209, 139),
        (245, 245, 67),
        (59, 142, 234),
        (214, 112, 214),
        (41, 184, 219),
        (255, 255, 255),
    ];
    if let Some(color) = colors[index] {
        return color;
    }
    let (r, g, b) = match index {
        0..=15 => ANSI[index],
        16..=231 => {
            let cube = index - 16;
            let channel = |level| if level == 0 { 0 } else { 55 + level as u8 * 40 };
            (channel(cube / 36), channel(cube / 6 % 6), channel(cube % 6))
        }
        232..=255 => {
            let shade = 8 + (index - 232) as u8 * 10;
            (shade, shade, shade)
        }
        257 => (24, 25, 29),
        _ => (229, 229, 229),
    };
    Rgb { r, g, b }
}

#[derive(Default)]
struct PointerState {
    terminal: std::sync::Weak<
        alacritty_terminal::sync::FairMutex<alacritty_terminal::Term<super::session::Listener>>,
    >,
    dragging: bool,
    pointer: Option<Point>,
    scroll_remainder: f32,
}

impl Metrics {
    fn cell_at(
        self,
        pointer: Point,
        bounds: Rectangle,
        columns: usize,
        lines: usize,
        offset: usize,
    ) -> (CellPoint, Side) {
        let column = ((pointer.x - bounds.x - PADDING) / self.width).max(0.0);
        let line =
            (((pointer.y - bounds.y - PADDING) / self.height).max(0.0) as usize).min(lines - 1);
        let index = (column as usize).min(columns - 1);
        let side = if column - index as f32 >= 0.5 {
            Side::Right
        } else {
            Side::Left
        };
        (
            CellPoint::new(Line(line as i32 - offset as i32), Column(index)),
            side,
        )
    }
}

fn paste_bytes(text: &str, bracketed: bool) -> Vec<u8> {
    // Escape cannot be included inside a bracketed paste since it can end the
    // paste before the user intended. Carriage returns normalize CRLF input.
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    if bracketed {
        let text = text.replace('\x1b', "");
        format!("\x1b[200~{text}\x1b[201~").into_bytes()
    } else {
        text.replace('\n', "\r").into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_pointer_with_padding_scrollback_and_cell_sides() {
        let metrics = Metrics {
            font_size: 16.0,
            width: 10.0,
            height: 20.0,
        };
        let bounds = Rectangle::new(Point::new(100.0, 50.0), Size::new(100.0, 100.0));
        assert_eq!(
            metrics.cell_at(Point::new(133.0, 83.0), bounds, 8, 4, 10),
            (CellPoint::new(Line(-9), Column(2)), Side::Right)
        );
        assert_eq!(
            metrics.cell_at(Point::new(0.0, 0.0), bounds, 8, 4, 10),
            (CellPoint::new(Line(-10), Column(0)), Side::Left)
        );
        assert_eq!(
            metrics.cell_at(Point::new(999.0, 999.0), bounds, 8, 4, 10),
            (CellPoint::new(Line(-7), Column(7)), Side::Right)
        );
    }

    #[test]
    fn selection_after_column_resize_uses_reflowed_cell_positions() {
        use alacritty_terminal::{
            Term, event::VoidListener, term::test::TermSize, vte::ansi::Processor,
        };
        let mut terminal = Term::new(Default::default(), &TermSize::new(8, 6), VoidListener);
        Processor::<alacritty_terminal::vte::ansi::StdSyncHandler>::new()
            .advance(&mut terminal, b"abcdefghijkl");
        let mut selection = Selection::new(
            SelectionType::Simple,
            CellPoint::new(Line(0), Column(2)),
            Side::Left,
        );
        selection.update(CellPoint::new(Line(1), Column(2)), Side::Right);
        terminal.selection = Some(selection);
        assert_eq!(terminal.selection_to_string().as_deref(), Some("cdefghijk"));
        terminal.resize(TermSize::new(5, 6));
        assert!(terminal.selection.is_none());
        let mut selection = Selection::new(
            SelectionType::Simple,
            CellPoint::new(Line(-1), Column(2)),
            Side::Left,
        );
        selection.update(CellPoint::new(Line(1), Column(0)), Side::Right);
        terminal.selection = Some(selection);
        assert_eq!(terminal.selection_to_string().as_deref(), Some("cdefghijk"));
    }

    #[test]
    fn paste_preserves_multiline_brackets_and_filters_escape_delimiters() {
        assert_eq!(paste_bytes("a\r\nb\rc", false), b"a\rb\rc");
        assert_eq!(
            paste_bytes("a\n\x1b[201~b", true),
            b"\x1b[200~a\n[201~b\x1b[201~"
        );
    }
}
