use crate::fuzzy::matching::find_match_positions;
use crate::fuzzy::FuzzyFinder;
use crate::tui::controls::handle_key_event;
use crossterm::{
    cursor,
    event::{self, Event},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, size, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

/// Configuration for the terminal user interface.
#[derive(Debug, Clone)]
pub struct TuiConfig {
    /// Whether to use fullscreen mode
    pub fullscreen: bool,
    /// Fixed height in lines (non-fullscreen mode)
    pub height: Option<u16>,
    /// Height as percentage of terminal (non-fullscreen mode)
    pub height_percentage: Option<f32>,
    /// Whether to show help/instructions text at the bottom
    pub show_help_text: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            fullscreen: true,
            height: None,
            height_percentage: None,
            show_help_text: true,
        }
    }
}

impl TuiConfig {
    /// Create a new TUI configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with a fixed height.
    pub fn with_height(height: u16) -> Self {
        Self {
            fullscreen: false,
            height: Some(height),
            height_percentage: None,
            show_help_text: true,
        }
    }

    /// Create a configuration with height as percentage of terminal.
    pub fn with_height_percentage(percentage: f32) -> Self {
        Self {
            fullscreen: false,
            height: None,
            height_percentage: Some(percentage),
            show_help_text: true,
        }
    }

    /// Create a fullscreen configuration.
    pub fn fullscreen() -> Self {
        Self {
            fullscreen: true,
            height: None,
            height_percentage: None,
            show_help_text: true,
        }
    }

    /// Calculate the actual height based on terminal size and configuration.
    pub fn calculate_height(&self, terminal_height: u16) -> u16 {
        if self.fullscreen {
            terminal_height
        } else if let Some(height) = self.height {
            height.min(terminal_height)
        } else if let Some(percentage) = self.height_percentage {
            let calculated = (terminal_height as f32 * percentage / 100.0) as u16;
            calculated.max(1).min(terminal_height)
        } else {
            terminal_height
        }
    }
}

/// Run the terminal user interface with default configuration.
pub fn run_tui(
    items: Vec<String>,
    multi_select: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    run_tui_with_config(items, multi_select, TuiConfig::default())
}

/// Run the terminal user interface with custom configuration.
pub fn run_tui_with_config(
    items: Vec<String>,
    multi_select: bool,
    config: TuiConfig,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    run_interactive_tui(items, multi_select, config)
}

fn run_interactive_tui(
    items: Vec<String>,
    multi_select: bool,
    config: TuiConfig,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    let _raw_mode = RawModeGuard::new().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut fuzzy_finder = FuzzyFinder::new(items, multi_select);
    fuzzy_finder.update_filter();
    let mut result = Vec::new();
    let mut exit = false;

    let fullscreen = config.fullscreen;
    let mut original_cursor = cursor::position()?;
            let (_term_width, term_height) = size()?;
    let tui_height = config.calculate_height(term_height);

    if fullscreen {
        execute!(stdout, EnterAlternateScreen, cursor::Hide, Clear(ClearType::All))?;
    } else {
        // If not enough space below, scroll the terminal down
        if original_cursor.1 + tui_height > term_height {
            let needed = (original_cursor.1 + tui_height).saturating_sub(term_height);
            for _ in 0..needed {
                writeln!(stdout)?;
            }
            stdout.flush()?;
            // After scrolling, we should draw at the bottom of the terminal
            original_cursor = (0, term_height.saturating_sub(tui_height));
        }
        // Always move to column 0 at the current line
        execute!(stdout, cursor::MoveTo(0, original_cursor.1))?;
        execute!(stdout, cursor::Hide)?;
    }

    while !exit {
        let (_term_width, term_height) = size()?;
        let tui_height = config.calculate_height(term_height);
        // Always reserve 1 line for prompt, 1 for result if possible, 1 for instructions
        let available_height = if tui_height > 2 {
            tui_height - 2 // 1 for prompt, 1 for instructions
        } else if tui_height == 2 {
            1 // Only room for prompt and one result
        } else {
            0 // Only room for prompt
        };

        // Draw TUI - always start at the original cursor position
        if fullscreen {
            execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        } else {
            for i in 0..tui_height.max(2) {
                execute!(stdout, cursor::MoveTo(0, original_cursor.1 + i), Clear(ClearType::CurrentLine))?;
            }
            execute!(stdout, cursor::MoveTo(0, original_cursor.1))?;
        }

        // Draw search prompt
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Search: "),
            ResetColor,
            Print(&fuzzy_finder.query)
        )?;


        if tui_height >= 2 && available_height > 0 {
            let visible_items = fuzzy_finder
                .filtered_items
                .iter()
                .take(available_height as usize);
            for (i, item) in visible_items.enumerate() {
                let y_pos = if fullscreen {
                    (i + 1) as u16
                } else {
                    original_cursor.1 + 1 + i as u16
                };
                execute!(stdout, cursor::MoveTo(0, y_pos))?;
                let is_cursor = i == fuzzy_finder.cursor_position;
                let is_selected = multi_select && {
                    if let Some(original_idx) = fuzzy_finder.items.iter().position(|x| x == item) {
                        fuzzy_finder.selected_indices.contains(&original_idx)
                    } else {
                        false
                    }
                };
                if multi_select {
                    if is_selected {
                        execute!(stdout, SetForegroundColor(Color::Green), Print("✓ "))?;
                    } else {
                        execute!(stdout, Print("  "))?;
                    }
                }
                draw_highlighted_item(
                    &mut stdout,
                    item,
                    &fuzzy_finder.query,
                    is_cursor,
                    is_selected,
                )?;
                execute!(stdout, ResetColor)?;
            }
        }
        if tui_height < 2 {
            let warning_y = if fullscreen {
                1
            } else {
                original_cursor.1 + 1
            };
            execute!(
                stdout,
                cursor::MoveTo(0, warning_y),
                SetForegroundColor(Color::Yellow),
                Print("Terminal too small. Please resize to continue..."),
                ResetColor
            )?;
        }

        // Draw instructions (always at the bottom of the TUI area)
        if config.show_help_text {
            let instructions_y = if fullscreen {
                tui_height.saturating_sub(1)
            } else {
                original_cursor.1 + tui_height.saturating_sub(1)
            };
            execute!(
                stdout,
                cursor::MoveTo(0, instructions_y),
                SetForegroundColor(Color::DarkGrey),
                Print("[↑/↓] Navigate  [Enter] Select  [Tab/Space] Toggle (multi)  [Esc/Ctrl+Q] Exit"),
                ResetColor
            )?;
        }
        stdout.flush()?;

        // Handle input
        match event::read()? {
            Event::Key(key_event) => {
                let action = handle_key_event(&key_event, &mut fuzzy_finder);
                match action {
                    crate::tui::controls::Action::Exit => exit = true,
                    crate::tui::controls::Action::Select(items) => {
                        result = items;
                        exit = true;
                    }
                    crate::tui::controls::Action::Continue => {}
                }
            }
            Event::Resize(_, _) => continue,
            _ => {}
        }
    }

    // Restore terminal
    if fullscreen {
        execute!(stdout, LeaveAlternateScreen)?;
    } else {
        for i in 0..config.calculate_height(size()?.1) {
            execute!(stdout, cursor::MoveTo(0, original_cursor.1 + i), Clear(ClearType::CurrentLine))?;
        }
        execute!(stdout, cursor::MoveTo(original_cursor.0, original_cursor.1), cursor::Show)?;
        stdout.flush()?;
    }

    if result.is_empty() && multi_select {
        result = fuzzy_finder.get_selected_items();
    }
    Ok(result)
}

fn draw_highlighted_item(
    stdout: &mut io::Stdout,
    item: &str,
    query: &str,
    is_cursor: bool,
    is_selected: bool,
) -> io::Result<()> {
    use crossterm::style::{Attribute, SetAttribute};
    let match_positions = find_match_positions(item, query);
    let mut match_set = std::collections::HashSet::new();
    for &pos in &match_positions {
        match_set.insert(pos);
    }
    let mut char_indices = item.char_indices().peekable();
    let mut char_pos = 0;
    if is_cursor {
        // Use reverse video for the selected row for high contrast
        execute!(stdout, SetAttribute(Attribute::Reverse))?;
    }
    while let Some((byte_idx, _ch)) = char_indices.next() {
        let is_match = match_set.contains(&char_pos);
        if is_match {
            // Use bold and underline for matched characters
            execute!(stdout, SetAttribute(Attribute::Bold), SetAttribute(Attribute::Underlined))?;
        } else if is_selected {
            // Use bold for selected (multi-select) but not matched
            execute!(stdout, SetAttribute(Attribute::Bold))?;
        } else {
            execute!(stdout, SetAttribute(Attribute::Reset))?;
        }
        let next_byte_idx = char_indices.peek().map(|(i, _)| *i).unwrap_or(item.len());
        execute!(stdout, Print(&item[byte_idx..next_byte_idx]))?;
        // Reset after each char
        execute!(stdout, SetAttribute(Attribute::Reset))?;
        char_pos += 1;
    }
    if is_cursor {
        execute!(stdout, SetAttribute(Attribute::NoReverse))?;
    }
    Ok(())
}

struct RawModeGuard;
impl RawModeGuard {
    fn new() -> std::io::Result<Self> {
        enable_raw_mode().map_err(std::io::Error::other)?;
        Ok(Self)
    }
}
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}
