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

fn draw_highlighted_item<W: Write>(
    stdout: &mut W,
    item: &str,
    query: &str,
    is_cursor: bool,
    is_selected: bool,
) -> io::Result<()> {
    use crossterm::style::{Attribute, SetAttribute, SetBackgroundColor, SetForegroundColor, Color, Print};
    let match_positions = find_match_positions(item, query);
    let mut match_set = std::collections::HashSet::new();
    for &pos in &match_positions {
        match_set.insert(pos);
    }
    let mut char_indices = item.char_indices().peekable();
    let mut char_pos = 0;

    if is_cursor {
        // Gruvbox soft highlight: dark grey background, yellow foreground, bold
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::Yellow),
            SetAttribute(Attribute::Bold)
        )?;
    }

    while let Some((byte_idx, _ch)) = char_indices.next() {
        let is_match = match_set.contains(&char_pos);
        if is_match {
            if is_cursor {
                // For selected rows, use a bright color that contrasts with dark grey background
                execute!(stdout, SetForegroundColor(Color::White), SetAttribute(Attribute::Bold), SetAttribute(Attribute::Underlined))?;
            } else {
                // For non-selected rows, use bold and underline
                execute!(stdout, SetAttribute(Attribute::Bold), SetAttribute(Attribute::Underlined))?;
            }
        } else if is_selected {
            execute!(stdout, SetAttribute(Attribute::Bold))?;
        }
        let next_byte_idx = char_indices.peek().map(|(i, _)| *i).unwrap_or(item.len());
        execute!(stdout, Print(&item[byte_idx..next_byte_idx]))?;
        
        // Reset attributes after each character to prevent bleeding
        if is_match {
            if is_cursor {
                // Reset to the row's base colors (dark grey background, yellow text)
                execute!(stdout, SetForegroundColor(Color::Yellow), SetAttribute(Attribute::Reset))?;
            } else {
                execute!(stdout, SetAttribute(Attribute::Reset))?;
            }
        } else if is_selected {
            execute!(stdout, SetAttribute(Attribute::Reset))?;
        }
        
        char_pos += 1;
    }

    // Only reset at the end of the line
    if is_cursor {
        execute!(
            stdout,
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset),
            SetAttribute(Attribute::Reset)
        )?;
    } else {
        execute!(stdout, SetAttribute(Attribute::Reset))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_draw_highlighted_item_cursor_highlighting() {
        // Test that cursor highlighting works
        let mut cursor = Cursor::new(Vec::new());
        let result = draw_highlighted_item(&mut cursor, "test", "", true, false);
        assert!(result.is_ok());
        
        // The cursor should have written some output
        let output = cursor.into_inner();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_draw_highlighted_item_no_cursor() {
        // Test that non-cursor items don't get highlighted
        let mut cursor = Cursor::new(Vec::new());
        let result = draw_highlighted_item(&mut cursor, "test", "", false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_draw_highlighted_item_with_matches() {
        // Test highlighting with query matches
        let mut cursor = Cursor::new(Vec::new());
        let result = draw_highlighted_item(&mut cursor, "test", "t", false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_draw_highlighted_item_selected() {
        // Test highlighting for selected items in multi-select
        let mut cursor = Cursor::new(Vec::new());
        let result = draw_highlighted_item(&mut cursor, "test", "", false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cursor_position_logic() {
        // Test that the cursor position logic works correctly
        let items = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
        let mut finder = FuzzyFinder::new(items, false);
        finder.update_filter();
        
        // Initial cursor position should be 0
        assert_eq!(finder.cursor_position, 0);
        
        // Move cursor down
        finder.move_cursor(1);
        assert_eq!(finder.cursor_position, 1);
        
        // Move cursor down again
        finder.move_cursor(1);
        assert_eq!(finder.cursor_position, 2);
        
        // Move cursor down (should wrap to 0)
        finder.move_cursor(1);
        assert_eq!(finder.cursor_position, 0);
        
        // Move cursor up (should wrap to 2)
        finder.move_cursor(-1);
        assert_eq!(finder.cursor_position, 2);
    }

    #[test]
    fn test_cursor_highlighting_logic() {
        // Test that the highlighting logic correctly identifies the cursor position
        let items = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
        let mut finder = FuzzyFinder::new(items, false);
        finder.update_filter();
        
        // Test that i == cursor_position logic works
        for i in 0..finder.filtered_items.len() {
            let is_cursor = i == finder.cursor_position;
            if i == 0 {
                assert!(is_cursor); // First item should be highlighted initially
            } else {
                assert!(!is_cursor); // Other items should not be highlighted
            }
        }
        
        // Move cursor and test again
        finder.move_cursor(1);
        for i in 0..finder.filtered_items.len() {
            let is_cursor = i == finder.cursor_position;
            if i == 1 {
                assert!(is_cursor); // Second item should be highlighted
            } else {
                assert!(!is_cursor); // Other items should not be highlighted
            }
        }
    }

    #[test]
    fn test_highlighting_colors_applied() {
        // Test that the highlighting colors are actually being applied
        let mut cursor = Cursor::new(Vec::new());
        
        // Test cursor highlighting
        let result = draw_highlighted_item(&mut cursor, "test", "", true, false);
        assert!(result.is_ok());
        
        let output = cursor.into_inner();
        // The output should contain color codes for white background and black text
        let output_str = String::from_utf8_lossy(&output);
        
        // Check that we have some output (indicating colors were applied)
        assert!(!output.is_empty());
        
        // Create a new cursor for non-cursor test
        let mut cursor2 = Cursor::new(Vec::new());
        let result2 = draw_highlighted_item(&mut cursor2, "test", "", false, false);
        assert!(result2.is_ok());
        
        let output2 = cursor2.into_inner();
        // Non-cursor output should be different (no background color)
        assert_ne!(output, output2);
    }
}
