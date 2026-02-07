//! Double-buffered screen rendering to eliminate flickering.
//!
//! This module provides a `ScreenBuffer` that accumulates all drawing operations
//! in memory, then renders the entire frame to the terminal in a single write.

use crossterm::style::Color;
use std::fmt::Write as FmtWrite;

/// Style attributes for a cell or text span
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style {
    /// Foreground color (None = default/reset)
    pub fg: Option<Color>,
    /// Background color (None = default/reset)
    pub bg: Option<Color>,
    /// Whether the text is bold
    pub bold: bool,
    /// Whether the text is underlined
    pub underline: bool,
}

impl Style {
    /// Create a new style with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set foreground color
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set background color
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Set bold
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set underline
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

/// A single cell in the screen buffer, representing one character position.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    /// The character to display
    pub ch: char,
    /// Foreground color (None = default/reset)
    pub fg: Option<Color>,
    /// Background color (None = default/reset)
    pub bg: Option<Color>,
    /// Whether the cell is bold
    pub bold: bool,
    /// Whether the cell is underlined
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
            bold: false,
            underline: false,
        }
    }
}

impl Cell {
    /// Create a new cell with just a character
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            ..Default::default()
        }
    }

    /// Create a cell with full styling
    pub fn styled(
        ch: char,
        fg: Option<Color>,
        bg: Option<Color>,
        bold: bool,
        underline: bool,
    ) -> Self {
        Self {
            ch,
            fg,
            bg,
            bold,
            underline,
        }
    }
}

/// A screen buffer that holds the entire frame in memory.
///
/// All drawing operations write to this buffer, and then the entire
/// frame is rendered to the terminal in one operation.
pub struct ScreenBuffer {
    cells: Vec<Cell>,
    width: u16,
    height: u16,
}

impl ScreenBuffer {
    /// Create a new screen buffer with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            cells: vec![Cell::default(); size],
            width,
            height,
        }
    }

    /// Clear the buffer, resetting all cells to default (spaces).
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    /// Resize the buffer if dimensions changed.
    pub fn resize(&mut self, width: u16, height: u16) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            let size = (width as usize) * (height as usize);
            self.cells = vec![Cell::default(); size];
        }
    }

    /// Get the buffer width.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get the buffer height.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Get the index into the cells vector for a given position.
    #[inline]
    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y as usize) * (self.width as usize) + (x as usize))
        } else {
            None
        }
    }

    /// Set a single cell at the given position.
    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(idx) = self.index(x, y) {
            self.cells[idx] = cell;
        }
    }

    /// Put a string at the given position with styling.
    /// Returns the number of characters actually written.
    #[allow(clippy::too_many_arguments)]
    pub fn put_str(
        &mut self,
        x: u16,
        y: u16,
        text: &str,
        fg: Option<Color>,
        bg: Option<Color>,
        bold: bool,
        underline: bool,
    ) -> u16 {
        let mut written = 0;
        for (i, ch) in text.chars().enumerate() {
            let cell_x = x.saturating_add(i as u16);
            if cell_x >= self.width {
                break;
            }
            self.set_cell(cell_x, y, Cell::styled(ch, fg, bg, bold, underline));
            written += 1;
        }
        written
    }

    /// Put a string with default styling (no colors, no attributes).
    pub fn put_str_plain(&mut self, x: u16, y: u16, text: &str) -> u16 {
        self.put_str(x, y, text, None, None, false, false)
    }

    /// Put a single character at the given position with styling.
    #[allow(clippy::too_many_arguments)]
    pub fn put_char(
        &mut self,
        x: u16,
        y: u16,
        ch: char,
        fg: Option<Color>,
        bg: Option<Color>,
        bold: bool,
        underline: bool,
    ) {
        self.set_cell(x, y, Cell::styled(ch, fg, bg, bold, underline));
    }

    /// Render the buffer to a string containing ANSI escape sequences.
    /// This produces the complete output that can be written to the terminal.
    pub fn render(&self, start_row: u16) -> String {
        let mut output = String::with_capacity((self.width as usize + 20) * self.height as usize);

        // Track current style state to minimize escape sequences
        let mut current_fg: Option<Color> = None;
        let mut current_bg: Option<Color> = None;
        let mut current_bold = false;
        let mut current_underline = false;

        for y in 0..self.height {
            // Move cursor to start of line
            let _ = write!(output, "\x1b[{};1H", start_row + y + 1);

            // Clear the line first
            let _ = write!(output, "\x1b[2K");

            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                let cell = &self.cells[idx];

                // Handle style changes
                let mut style_changed = false;

                // Check if we need to reset (going from styled to unstyled)
                let needs_reset = (current_bold && !cell.bold)
                    || (current_underline && !cell.underline)
                    || (current_fg.is_some() && cell.fg.is_none())
                    || (current_bg.is_some() && cell.bg.is_none());

                if needs_reset {
                    let _ = write!(output, "\x1b[0m");
                    current_fg = None;
                    current_bg = None;
                    current_bold = false;
                    current_underline = false;
                    style_changed = true;
                }

                // Apply bold if needed
                if cell.bold && !current_bold {
                    let _ = write!(output, "\x1b[1m");
                    current_bold = true;
                    style_changed = true;
                }

                // Apply underline if needed
                if cell.underline && !current_underline {
                    let _ = write!(output, "\x1b[4m");
                    current_underline = true;
                    style_changed = true;
                }

                // Apply foreground color if changed
                if cell.fg != current_fg && cell.fg.is_some() {
                    if let Some(color) = cell.fg {
                        write_fg_color(&mut output, color);
                        current_fg = cell.fg;
                        style_changed = true;
                    }
                }

                // Apply background color if changed
                if cell.bg != current_bg && cell.bg.is_some() {
                    if let Some(color) = cell.bg {
                        write_bg_color(&mut output, color);
                        current_bg = cell.bg;
                        style_changed = true;
                    }
                }

                let _ = style_changed; // Suppress warning
                output.push(cell.ch);
            }
        }

        // Reset all attributes at the end
        let _ = write!(output, "\x1b[0m");

        output
    }

    /// Render the buffer for fullscreen mode (starting at row 0).
    pub fn render_fullscreen(&self) -> String {
        let mut output = String::with_capacity((self.width as usize + 20) * self.height as usize);

        // Move to top-left and clear screen
        let _ = write!(output, "\x1b[H\x1b[2J");

        // Track current style state
        let mut current_fg: Option<Color> = None;
        let mut current_bg: Option<Color> = None;
        let mut current_bold = false;
        let mut current_underline = false;

        for y in 0..self.height {
            // Move cursor to start of line
            let _ = write!(output, "\x1b[{};1H", y + 1);

            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                let cell = &self.cells[idx];

                // Check if we need to reset
                let needs_reset = (current_bold && !cell.bold)
                    || (current_underline && !cell.underline)
                    || (current_fg.is_some() && cell.fg.is_none())
                    || (current_bg.is_some() && cell.bg.is_none());

                if needs_reset {
                    let _ = write!(output, "\x1b[0m");
                    current_fg = None;
                    current_bg = None;
                    current_bold = false;
                    current_underline = false;
                }

                if cell.bold && !current_bold {
                    let _ = write!(output, "\x1b[1m");
                    current_bold = true;
                }

                if cell.underline && !current_underline {
                    let _ = write!(output, "\x1b[4m");
                    current_underline = true;
                }

                if cell.fg != current_fg && cell.fg.is_some() {
                    if let Some(color) = cell.fg {
                        write_fg_color(&mut output, color);
                        current_fg = cell.fg;
                    }
                }

                if cell.bg != current_bg && cell.bg.is_some() {
                    if let Some(color) = cell.bg {
                        write_bg_color(&mut output, color);
                        current_bg = cell.bg;
                    }
                }

                output.push(cell.ch);
            }
        }

        // Reset all attributes at the end
        let _ = write!(output, "\x1b[0m");

        output
    }
}

/// Write foreground color escape sequence
fn write_fg_color(output: &mut String, color: Color) {
    match color {
        Color::Black => {
            let _ = write!(output, "\x1b[30m");
        }
        Color::DarkGrey => {
            let _ = write!(output, "\x1b[90m");
        }
        Color::Red => {
            let _ = write!(output, "\x1b[31m");
        }
        Color::DarkRed => {
            let _ = write!(output, "\x1b[31m");
        }
        Color::Green => {
            let _ = write!(output, "\x1b[32m");
        }
        Color::DarkGreen => {
            let _ = write!(output, "\x1b[32m");
        }
        Color::Yellow => {
            let _ = write!(output, "\x1b[33m");
        }
        Color::DarkYellow => {
            let _ = write!(output, "\x1b[33m");
        }
        Color::Blue => {
            let _ = write!(output, "\x1b[34m");
        }
        Color::DarkBlue => {
            let _ = write!(output, "\x1b[34m");
        }
        Color::Magenta => {
            let _ = write!(output, "\x1b[35m");
        }
        Color::DarkMagenta => {
            let _ = write!(output, "\x1b[35m");
        }
        Color::Cyan => {
            let _ = write!(output, "\x1b[36m");
        }
        Color::DarkCyan => {
            let _ = write!(output, "\x1b[36m");
        }
        Color::White => {
            let _ = write!(output, "\x1b[37m");
        }
        Color::Grey => {
            let _ = write!(output, "\x1b[37m");
        }
        Color::Rgb { r, g, b } => {
            let _ = write!(output, "\x1b[38;2;{};{};{}m", r, g, b);
        }
        Color::AnsiValue(v) => {
            let _ = write!(output, "\x1b[38;5;{}m", v);
        }
        Color::Reset => {
            let _ = write!(output, "\x1b[39m");
        }
    }
}

/// Write background color escape sequence
fn write_bg_color(output: &mut String, color: Color) {
    match color {
        Color::Black => {
            let _ = write!(output, "\x1b[40m");
        }
        Color::DarkGrey => {
            let _ = write!(output, "\x1b[100m");
        }
        Color::Red => {
            let _ = write!(output, "\x1b[41m");
        }
        Color::DarkRed => {
            let _ = write!(output, "\x1b[41m");
        }
        Color::Green => {
            let _ = write!(output, "\x1b[42m");
        }
        Color::DarkGreen => {
            let _ = write!(output, "\x1b[42m");
        }
        Color::Yellow => {
            let _ = write!(output, "\x1b[43m");
        }
        Color::DarkYellow => {
            let _ = write!(output, "\x1b[43m");
        }
        Color::Blue => {
            let _ = write!(output, "\x1b[44m");
        }
        Color::DarkBlue => {
            let _ = write!(output, "\x1b[44m");
        }
        Color::Magenta => {
            let _ = write!(output, "\x1b[45m");
        }
        Color::DarkMagenta => {
            let _ = write!(output, "\x1b[45m");
        }
        Color::Cyan => {
            let _ = write!(output, "\x1b[46m");
        }
        Color::DarkCyan => {
            let _ = write!(output, "\x1b[46m");
        }
        Color::White => {
            let _ = write!(output, "\x1b[47m");
        }
        Color::Grey => {
            let _ = write!(output, "\x1b[47m");
        }
        Color::Rgb { r, g, b } => {
            let _ = write!(output, "\x1b[48;2;{};{};{}m", r, g, b);
        }
        Color::AnsiValue(v) => {
            let _ = write!(output, "\x1b[48;5;{}m", v);
        }
        Color::Reset => {
            let _ = write!(output, "\x1b[49m");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.ch, ' ');
        assert_eq!(cell.fg, None);
        assert_eq!(cell.bg, None);
        assert!(!cell.bold);
        assert!(!cell.underline);
    }

    #[test]
    fn test_cell_new() {
        let cell = Cell::new('x');
        assert_eq!(cell.ch, 'x');
        assert_eq!(cell.fg, None);
    }

    #[test]
    fn test_cell_styled() {
        let cell = Cell::styled('A', Some(Color::Red), Some(Color::Blue), true, true);
        assert_eq!(cell.ch, 'A');
        assert_eq!(cell.fg, Some(Color::Red));
        assert_eq!(cell.bg, Some(Color::Blue));
        assert!(cell.bold);
        assert!(cell.underline);
    }

    #[test]
    fn test_buffer_new() {
        let buffer = ScreenBuffer::new(80, 24);
        assert_eq!(buffer.width(), 80);
        assert_eq!(buffer.height(), 24);
        assert_eq!(buffer.cells.len(), 80 * 24);
    }

    #[test]
    fn test_buffer_clear() {
        let mut buffer = ScreenBuffer::new(10, 10);
        buffer.put_str(0, 0, "test", Some(Color::Red), None, true, false);
        buffer.clear();

        // All cells should be default after clear
        for cell in &buffer.cells {
            assert_eq!(*cell, Cell::default());
        }
    }

    #[test]
    fn test_buffer_resize() {
        let mut buffer = ScreenBuffer::new(10, 10);
        buffer.put_str(0, 0, "test", None, None, false, false);

        buffer.resize(20, 20);
        assert_eq!(buffer.width(), 20);
        assert_eq!(buffer.height(), 20);
        assert_eq!(buffer.cells.len(), 400);
    }

    #[test]
    fn test_buffer_put_str() {
        let mut buffer = ScreenBuffer::new(20, 10);
        let written = buffer.put_str(0, 0, "hello", Some(Color::Green), None, false, false);

        assert_eq!(written, 5);
        assert_eq!(buffer.cells[0].ch, 'h');
        assert_eq!(buffer.cells[1].ch, 'e');
        assert_eq!(buffer.cells[4].ch, 'o');
        assert_eq!(buffer.cells[0].fg, Some(Color::Green));
    }

    #[test]
    fn test_buffer_put_str_truncate() {
        let mut buffer = ScreenBuffer::new(5, 1);
        let written = buffer.put_str(0, 0, "hello world", None, None, false, false);

        assert_eq!(written, 5);
        assert_eq!(buffer.cells[4].ch, 'o');
    }

    #[test]
    fn test_buffer_put_char() {
        let mut buffer = ScreenBuffer::new(10, 10);
        buffer.put_char(
            5,
            5,
            'X',
            Some(Color::Yellow),
            Some(Color::Blue),
            true,
            true,
        );

        let idx = 5 * 10 + 5;
        assert_eq!(buffer.cells[idx].ch, 'X');
        assert_eq!(buffer.cells[idx].fg, Some(Color::Yellow));
        assert_eq!(buffer.cells[idx].bg, Some(Color::Blue));
        assert!(buffer.cells[idx].bold);
        assert!(buffer.cells[idx].underline);
    }

    #[test]
    fn test_buffer_out_of_bounds() {
        let mut buffer = ScreenBuffer::new(10, 10);
        // These should not panic
        buffer.set_cell(100, 100, Cell::new('x'));
        buffer.put_str(100, 0, "test", None, None, false, false);
    }

    #[test]
    fn test_render_contains_escape_sequences() {
        let mut buffer = ScreenBuffer::new(10, 2);
        buffer.put_str(0, 0, "hello", Some(Color::Red), None, true, false);

        let output = buffer.render(0);

        // Should contain cursor positioning
        assert!(output.contains("\x1b["));
        // Should contain the text
        assert!(output.contains("hello"));
        // Should end with reset
        assert!(output.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_render_fullscreen() {
        let mut buffer = ScreenBuffer::new(10, 2);
        buffer.put_str(0, 0, "test", None, None, false, false);

        let output = buffer.render_fullscreen();

        // Should start with home and clear
        assert!(output.starts_with("\x1b[H\x1b[2J"));
        assert!(output.contains("test"));
    }
}
