use crate::fuzzy::matching::find_match_positions;
use crate::fuzzy::FuzzyFinder;
use crate::tui::controls::handle_key_event;
use crossterm::{
    cursor,
    event::{self, Event},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
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
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            fullscreen: true,
            height: None,
            height_percentage: None,
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
        }
    }

    /// Create a configuration with height as percentage of terminal.
    pub fn with_height_percentage(percentage: f32) -> Self {
        Self {
            fullscreen: false,
            height: None,
            height_percentage: Some(percentage),
        }
    }

    /// Create a fullscreen configuration.
    pub fn fullscreen() -> Self {
        Self {
            fullscreen: true,
            height: None,
            height_percentage: None,
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
            calculated.min(terminal_height)
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
    let _raw_mode = RawModeGuard::new().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    let mut fuzzy_finder = FuzzyFinder::new(items, multi_select);
    fuzzy_finder.update_filter();
    let mut result = Vec::new();
    let mut exit = false;

    // Hide cursor and clear screen
    execute!(stdout, cursor::Hide, Clear(ClearType::All))?;

    while !exit {
        // Check current terminal size
        let (_term_width, term_height) = crossterm::terminal::size()?;
        let tui_height = config.calculate_height(term_height);

        // Check if terminal is now large enough
        if tui_height < 2 {
            // Terminal is too small, show a message and wait for resize
            execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
            execute!(
                stdout,
                cursor::MoveTo(0, 0),
                SetForegroundColor(Color::Yellow),
                Print("Terminal too small. Please resize to continue..."),
                ResetColor
            )?;
            stdout.flush()?;

            // Wait for either a key press or terminal resize
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
                Event::Resize(_, _) => {
                    // Terminal was resized, redraw on next iteration
                    continue;
                }
                _ => {
                    // Ignore other events
                }
            }
            continue;
        }

        // Clear screen and move to top-left
        execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;

        // Draw search prompt at column 0
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Yellow),
            Print("Search: "),
            ResetColor,
            Print(&fuzzy_finder.query)
        )?;

        // Draw items with proper highlighting
        let available_height = tui_height.saturating_sub(3); // Safe subtraction
        let visible_items = fuzzy_finder
            .filtered_items
            .iter()
            .take(available_height as usize);

        for (i, item) in visible_items.enumerate() {
            let y_pos = (i + 1) as u16; // Start from line 1 (after search prompt)
            let is_cursor = i == fuzzy_finder.cursor_position;
            let is_selected = multi_select && {
                if let Some(original_idx) = fuzzy_finder.items.iter().position(|x| x == item) {
                    fuzzy_finder.selected_indices.contains(&original_idx)
                } else {
                    false
                }
            };

            // Move to the correct line and column 0
            execute!(stdout, cursor::MoveTo(0, y_pos))?;

            // Draw selection indicator for multi-select
            if multi_select {
                if is_selected {
                    execute!(stdout, SetForegroundColor(Color::Green), Print("✓ "))?;
                } else {
                    execute!(stdout, Print("  "))?;
                }
            }

            // Draw the item with highlighting
            draw_highlighted_item(
                &mut stdout,
                item,
                &fuzzy_finder.query,
                is_cursor,
                is_selected,
            )?;
            execute!(stdout, ResetColor)?;
        }

        // Draw instructions at the bottom
        let instructions_y = available_height + 1;
        execute!(
            stdout,
            cursor::MoveTo(0, instructions_y),
            SetForegroundColor(Color::DarkGrey),
            Print("[↑/↓] Navigate  [Enter] Select  [Tab/Space] Toggle (multi)  [Esc/Ctrl+Q] Exit"),
            ResetColor
        )?;

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
            Event::Resize(_, _) => {
                // Terminal was resized, redraw on next iteration
                continue;
            }
            _ => {
                // Ignore other events
            }
        }
    }

    // Restore cursor and clear screen
    execute!(stdout, cursor::Show, Clear(ClearType::All))?;
    disable_raw_mode()?;

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
    let match_positions = find_match_positions(item, query);
    let mut match_set = std::collections::HashSet::new();
    for &pos in &match_positions {
        match_set.insert(pos);
    }
    let mut char_indices = item.char_indices().peekable();
    let mut char_pos = 0;
    while let Some((byte_idx, _ch)) = char_indices.next() {
        // Determine if this char is a match
        let is_match = match_set.contains(&char_pos);
        // Set background for cursor/selection
        if is_cursor {
            execute!(stdout, SetBackgroundColor(Color::DarkGrey))?;
        }
        // Set highlight for match
        if is_match {
            execute!(
                stdout,
                SetBackgroundColor(Color::Yellow),
                SetForegroundColor(Color::Black)
            )?;
        } else if is_selected {
            execute!(stdout, SetForegroundColor(Color::Green))?;
        } else {
            execute!(stdout, ResetColor)?;
            if is_cursor {
                execute!(stdout, SetBackgroundColor(Color::DarkGrey))?;
            }
        }
        // Print the character
        let next_byte_idx = char_indices.peek().map(|(i, _)| *i).unwrap_or(item.len());
        execute!(stdout, Print(&item[byte_idx..next_byte_idx]))?;
        // Reset color after each char
        execute!(stdout, ResetColor)?;
        char_pos += 1;
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
