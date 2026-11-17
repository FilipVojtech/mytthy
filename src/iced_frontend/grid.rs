use super::session::{GridSize, Terminal};
use alacritty_terminal::{
    term::{RenderableCursor, cell::Cell, cell::Flags, color::Colors},
    vte::ansi::{Color as AnsiColor, CursorShape, NamedColor, Rgb},
};
use iced::{
    Color, Font, Length, Pixels, Point, Rectangle, Renderer, Size, Theme,
    advanced::{
        Layout, Widget, layout, mouse,
        renderer::{Quad, Renderer as _},
        text::{self, Paragraph, Renderer as _},
        widget::Tree,
    },
    alignment, font,
};

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

pub(super) struct Grid<'a> {
    pub metrics: Metrics,
    pub terminal: &'a Terminal,
    pub focused: bool,
}

impl<Message> Widget<Message, Theme, Renderer> for Grid<'_> {
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
        let (cells, columns, cursor, offset, colors) = {
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
                    renderer, bounds, cells, line, cursor, offset, &colors, background,
                );
            }
        });
    }
}

impl Grid<'_> {
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
            let style = CellStyle::new(cell, colors, cursor_shape);
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
                        || CellStyle::new(next, colors, CursorShape::Hidden) != style
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
    fn new(cell: &Cell, colors: &Colors, cursor: CursorShape) -> Self {
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
