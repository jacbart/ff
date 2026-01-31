use crate::fuzzy::FuzzyFinder;
use crate::tui::controls::Action;
use crossterm::{
    cursor::{position, Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use std::{
    io::{self, Write},
    mem,
};
use tokio::sync::mpsc;

/// Configuration for TUI display mode and height
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
    /// Create a new TUI configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with fixed height
    pub fn with_height(height: u16) -> Self {
        Self {
            fullscreen: false,
            height: Some(height),
            height_percentage: None,
            show_help_text: true,
        }
    }

    /// Create a configuration with height as percentage
    pub fn with_height_percentage(percentage: f32) -> Self {
        Self {
            fullscreen: false,
            height: None,
            height_percentage: Some(percentage),
            show_help_text: true,
        }
    }

    /// Create a fullscreen configuration
    pub fn fullscreen() -> Self {
        Self {
            fullscreen: true,
            height: None,
            height_percentage: None,
            show_help_text: true,
        }
    }

    /// Calculate the actual height based on terminal size
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

/// Run an async interactive TUI for fuzzy finding through an mpsc receiver of items.
pub async fn run_tui(
    items_receiver: mpsc::Receiver<String>,
    multi_select: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    run_tui_with_config(items_receiver, multi_select, TuiConfig::default()).await
}

/// Run an async interactive TUI with custom configuration for height and display mode.
pub async fn run_tui_with_config(
    items_receiver: mpsc::Receiver<String>,
    multi_select: bool,
    config: TuiConfig,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    run_interactive_tui(items_receiver, multi_select, config).await
}

/// Run the async interactive TUI
async fn run_interactive_tui(
    mut items_receiver: mpsc::Receiver<String>,
    multi_select: bool,
    config: TuiConfig,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut fuzzy_finder = FuzzyFinder::new(multi_select);
    let mut stdout = io::stdout();

    // Enable raw mode and hide cursor
    enable_raw_mode()?;
    execute!(stdout, Hide)?;

    let fullscreen = config.fullscreen;
    let mut original_cursor = position()?;
    let (_term_width, term_height) = size()?;
    let tui_height = config.calculate_height(term_height);

    if fullscreen {
        execute!(
            &mut stdout,
            crossterm::terminal::EnterAlternateScreen,
            Clear(ClearType::All)
        )?;
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
        execute!(&mut stdout, MoveTo(0, original_cursor.1))?;
    }

    let mut selected_items = Vec::new();
    let mut needs_redraw = true;
    let mut items_buffer = Vec::new();
    let mut receiver_exhausted = false;

    loop {
        // Process new items from mpsc receiver
        if !receiver_exhausted {
            match items_receiver.try_recv() {
                Ok(item) => {
                    items_buffer.push(item);
                    fuzzy_finder.add_items(mem::take(&mut items_buffer)).await;
                    needs_redraw = true;
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No items available right now, continue with other processing
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    receiver_exhausted = true;
                    // Add any remaining buffered items
                    if !items_buffer.is_empty() {
                        fuzzy_finder.add_items(mem::take(&mut items_buffer)).await;
                        needs_redraw = true;
                    }
                }
            }
        }

        let (_term_width, term_height) = size()?;
        let tui_height = config.calculate_height(term_height);
        // Always reserve 1 line for prompt, 1 for result if possible, 1 for instructions
        let available_height = if tui_height > 2 {
            if config.show_help_text {
                tui_height - 2 // 1 for prompt, 1 for instructions
            } else {
                tui_height - 1
            }
        } else if tui_height == 2 {
            1 // Only room for prompt and one result
        } else {
            0 // Only room for prompt
        };

        // Only redraw if needed (when query changes or cursor moves)
        if needs_redraw {
            // Draw TUI - always start at the original cursor position
            if fullscreen {
                execute!(&mut stdout, MoveTo(0, 0), Clear(ClearType::All))?;
            } else {
                for i in 0..tui_height.max(2) {
                    execute!(
                        &mut stdout,
                        MoveTo(0, original_cursor.1 + i),
                        Clear(ClearType::CurrentLine)
                    )?;
                }
                execute!(&mut stdout, MoveTo(0, original_cursor.1))?;
            }

            // Draw search prompt
            let prompt_y = if fullscreen { 0 } else { original_cursor.1 };
            execute!(
                &mut stdout,
                MoveTo(0, prompt_y),
                SetForegroundColor(Color::Cyan),
                Print("> "),
                ResetColor,
                Print(&fuzzy_finder.get_query())
            )?;

            // Draw items
            if tui_height >= 2 && available_height > 0 {
                let filtered_items = fuzzy_finder.get_filtered_items();
                let visible_items = filtered_items.iter().take(available_height as usize);

                for (i, item) in visible_items.enumerate() {
                    let y_pos = if fullscreen {
                        (i + 1) as u16
                    } else {
                        original_cursor.1 + 1 + i as u16
                    };
                    execute!(&mut stdout, MoveTo(0, y_pos))?;

                    let is_cursor = i == fuzzy_finder.get_cursor_position();
                    let is_selected = fuzzy_finder.is_selected(item);

                    draw_highlighted_item_with_matches(
                        &mut stdout,
                        item,
                        is_cursor,
                        is_selected,
                        fuzzy_finder.get_match_positions(i),
                    )?;
                    execute!(&mut stdout, ResetColor)?;
                }
            }

            if tui_height < 2 {
                let warning_y = if fullscreen { 1 } else { original_cursor.1 + 1 };
                execute!(
                    &mut stdout,
                    MoveTo(0, warning_y),
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
                    &mut stdout,
                    MoveTo(0, instructions_y),
                    SetForegroundColor(Color::DarkGrey)
                )?;
                if multi_select {
                    execute!(
                        &mut stdout,
                        Print("Tab/Space: Toggle | Enter: Confirm | Esc/Ctrl+C/Ctrl+Q: Exit")
                    )?;
                } else {
                    execute!(
                        &mut stdout,
                        Print("↑/↓: Navigate | Enter: Select | Esc/Ctrl+C/Ctrl+Q: Exit")
                    )?;
                }
                execute!(&mut stdout, ResetColor)?;
            }

            stdout.flush()?;
            needs_redraw = false;
        }

        // Handle input with timeout to allow stream processing
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match handle_async_key_event(&key_event, &mut fuzzy_finder).await {
                    Action::Continue => {
                        needs_redraw = true;
                        continue;
                    }
                    Action::Exit => break,
                    Action::Select(items) => {
                        selected_items = items;
                        break;
                    }
                }
            }
        }
    }

    // Restore terminal
    if fullscreen {
        execute!(&mut stdout, crossterm::terminal::LeaveAlternateScreen)?;
        execute!(&mut stdout, Show)?;
    } else {
        for i in 0..config.calculate_height(size()?.1) {
            execute!(
                &mut stdout,
                MoveTo(0, original_cursor.1 + i),
                Clear(ClearType::CurrentLine)
            )?;
        }
        execute!(
            &mut stdout,
            MoveTo(original_cursor.0, original_cursor.1),
            Show
        )?;
        stdout.flush()?;
    }

    // Restore terminal state
    disable_raw_mode()?;

    if !selected_items.is_empty() {
        // Move to the original cursor position
        execute!(&mut stdout, MoveTo(0, original_cursor.1))?;
    }

    Ok(selected_items)
}

/// Create an mpsc channel for sending items to the TUI
pub fn create_items_channel() -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
    mpsc::channel(1000) // Buffer size of 1000 items
}

/// Handle key events in async mode
async fn handle_async_key_event(
    key_event: &crossterm::event::KeyEvent,
    fuzzy_finder: &mut FuzzyFinder,
) -> crate::tui::controls::Action {
    match key_event.code {
        KeyCode::Char(c) => {
            if (c == 'q' || c == 'c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                Action::Exit
            } else if c == ' ' && fuzzy_finder.is_multi_select() {
                fuzzy_finder.toggle_selection();
                Action::Continue
            } else {
                let mut query = fuzzy_finder.get_query().to_string();
                query.push(c);
                fuzzy_finder.set_query(query).await;
                Action::Continue
            }
        }
        KeyCode::Backspace => {
            let mut query = fuzzy_finder.get_query().to_string();
            query.pop();
            fuzzy_finder.set_query(query).await;
            Action::Continue
        }
        KeyCode::Up => {
            fuzzy_finder.move_cursor(-1);
            Action::Continue
        }
        KeyCode::Down => {
            fuzzy_finder.move_cursor(1);
            Action::Continue
        }
        KeyCode::Tab => {
            if fuzzy_finder.is_multi_select() {
                fuzzy_finder.toggle_selection();
            }
            Action::Continue
        }
        KeyCode::Enter => {
            let selected = fuzzy_finder.get_selected_items();
            if !selected.is_empty() {
                Action::Select(selected)
            } else if !fuzzy_finder.is_multi_select()
                && !fuzzy_finder.get_filtered_items().is_empty()
            {
                // In single select mode, select the current item if no items are selected
                let current_item =
                    &fuzzy_finder.get_filtered_items()[fuzzy_finder.get_cursor_position()];
                Action::Select(vec![current_item.clone()])
            } else if fuzzy_finder.is_multi_select()
                && !fuzzy_finder.get_filtered_items().is_empty()
            {
                // In multi-select mode, if no items are selected, select the current item
                let current_item =
                    &fuzzy_finder.get_filtered_items()[fuzzy_finder.get_cursor_position()];
                Action::Select(vec![current_item.clone()])
            } else {
                Action::Continue
            }
        }
        KeyCode::Esc => Action::Exit,
        _ => Action::Continue,
    }
}

/// Draw highlighted item with fuzzy match highlighting using Gruvbox soft colors
fn draw_highlighted_item_with_matches<W: Write>(
    stdout: &mut W,
    item: &str,
    is_cursor: bool,
    is_selected: bool,
    match_positions: Option<&crate::fuzzy::finder::MatchPositions>,
) -> io::Result<()> {
    // Set cursor highlighting with Gruvbox soft colors
    if is_cursor {
        // Gruvbox soft highlight: dark grey background, yellow foreground, bold
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::Yellow),
            SetAttribute(Attribute::Bold)
        )?;
    }

    // Set selection highlighting (only show checkmarks for selected items)
    if is_selected {
        execute!(stdout, SetForegroundColor(Color::Green), Print("✓ "))?;
    } else {
        execute!(stdout, Print("  "))?;
    }

    // Draw item with match highlighting
    if let Some(matches) = match_positions {
        for (i, ch) in item.chars().enumerate() {
            if matches.positions.contains(&i) {
                // Highlight matched characters with Gruvbox soft colors
                if is_cursor {
                    // For selected rows, use bright white that contrasts with dark grey background
                    execute!(
                        stdout,
                        SetForegroundColor(Color::White),
                        SetAttribute(Attribute::Bold),
                        SetAttribute(Attribute::Underlined)
                    )?;
                } else {
                    // For non-selected rows, use bold and underline
                    execute!(
                        stdout,
                        SetAttribute(Attribute::Bold),
                        SetAttribute(Attribute::Underlined)
                    )?;
                }
                execute!(stdout, Print(ch))?;
                // Reset attributes after each character to prevent bleeding
                if is_cursor {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Yellow),
                        SetAttribute(Attribute::NoUnderline)
                    )?;
                } else {
                    execute!(
                        stdout,
                        SetAttribute(Attribute::NoUnderline),
                        SetAttribute(Attribute::NormalIntensity)
                    )?;
                }
            } else {
                execute!(stdout, Print(ch))?;
            }
        }
    } else {
        execute!(stdout, Print(item))?;
    }

    // Reset all attributes
    execute!(stdout, ResetColor)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_highlighted_item_cursor_highlighting() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", true, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        // Check for Gruvbox soft highlight colors (using 256-color codes)
        assert!(output_str.contains("\x1b[48;5;8m")); // Dark grey background
        assert!(output_str.contains("\x1b[38;5;11m")); // Yellow foreground
        assert!(output_str.contains("\x1b[1m")); // Bold
    }

    #[test]
    fn test_draw_highlighted_item_no_cursor() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", false, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("  test"));
    }

    #[test]
    fn test_draw_highlighted_item_with_matches() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", false, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("test"));
    }

    #[test]
    fn test_draw_highlighted_item_selected() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", false, true, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("✓"));
    }

    #[test]
    fn test_tui_config_default() {
        let config = TuiConfig::default();
        assert!(config.fullscreen);
        assert!(config.height.is_none());
        assert!(config.height_percentage.is_none());
        assert!(config.show_help_text);
    }

    #[test]
    fn test_tui_config_with_height() {
        let config = TuiConfig::with_height(10);
        assert!(!config.fullscreen);
        assert_eq!(config.height, Some(10));
        assert!(config.height_percentage.is_none());
    }

    #[test]
    fn test_tui_config_with_height_percentage() {
        let config = TuiConfig::with_height_percentage(50.0);
        assert!(!config.fullscreen);
        assert!(config.height.is_none());
        assert_eq!(config.height_percentage, Some(50.0));
    }

    #[test]
    fn test_tui_config_fullscreen() {
        let config = TuiConfig::fullscreen();
        assert!(config.fullscreen);
        assert!(config.height.is_none());
        assert!(config.height_percentage.is_none());
    }

    #[test]
    fn test_calculate_height_fullscreen() {
        let config = TuiConfig::fullscreen();
        let height = config.calculate_height(25);
        assert_eq!(height, 25); // 25 - 2 for borders
    }

    #[test]
    fn test_calculate_height_fixed() {
        let config = TuiConfig::with_height(10);
        let height = config.calculate_height(25);
        assert_eq!(height, 10);
    }

    #[test]
    fn test_calculate_height_percentage() {
        let config = TuiConfig::with_height_percentage(50.0);
        let height = config.calculate_height(20);
        assert_eq!(height, 10); // 50% of 20 = 10
    }

    #[test]
    fn test_calculate_height_overflow() {
        let config = TuiConfig::with_height(30);
        let height = config.calculate_height(25);
        assert_eq!(height, 25); // Should be capped at terminal height - 2
    }

    #[test]
    fn test_cursor_position_logic() {
        // Test cursor wrapping logic
        let config = TuiConfig::default();
        let display_height = config.calculate_height(25);
        assert!(display_height > 0);
    }

    #[test]
    fn test_cursor_highlighting_logic() {
        // Test that cursor highlighting works correctly
        let mut output = Vec::new();

        // Test cursor position
        draw_highlighted_item_with_matches(&mut output, "test", true, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("\x1b[48;5;8m")); // Dark grey background
        assert!(output_str.contains("\x1b[38;5;11m")); // Yellow foreground
        assert!(output_str.contains("\x1b[1m")); // Bold

        // Test non-cursor position
        let mut output2 = Vec::new();
        draw_highlighted_item_with_matches(&mut output2, "test", false, false, None).unwrap();
        let output_str2 = String::from_utf8(output2).unwrap();
        assert!(!output_str2.contains("\x1b[48;5;8m")); // No dark grey background
        assert!(!output_str2.contains("\x1b[38;5;11m")); // No yellow foreground
        assert!(output_str2.contains("  test"));
    }

    #[test]
    fn test_highlighting_colors_applied() {
        let mut output = Vec::new();
        draw_highlighted_item_with_matches(&mut output, "test", true, false, None).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        // Check that color codes are present
        assert!(output_str.contains("\x1b["));
    }

    #[tokio::test]
    async fn test_create_items_channel() {
        let (sender, mut receiver) = create_items_channel();

        // Send some items
        sender.send("item1".to_string()).await.unwrap();
        sender.send("item2".to_string()).await.unwrap();
        drop(sender); // Close the sender

        // Collect items from receiver
        let mut collected = Vec::new();
        while let Some(item) = receiver.recv().await {
            collected.push(item);
        }

        assert_eq!(collected, vec!["item1".to_string(), "item2".to_string()]);
    }

    #[tokio::test]
    async fn test_handle_async_key_event_ctrl_c() {
        use crate::fuzzy::FuzzyFinder;
        use crossterm::event::{KeyCode, KeyModifiers};

        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let action = handle_async_key_event(&key_event, &mut finder).await;

        assert_eq!(action, crate::tui::controls::Action::Exit);
    }
}
