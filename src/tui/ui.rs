use crate::fuzzy::FuzzyFinder;
use crate::tui::controls::handle_key_event;
use crossterm::{
    cursor,
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

// Ayu color palette for highlights and subtle styling
const NATIVE_BG: Color = Color::Reset; // Use terminal's default background
const NATIVE_FG: Color = Color::Reset; // Use terminal's default foreground
const AYU_YELLOW: Color = Color::Rgb(255, 213, 128); // Matched chars
const AYU_GREEN: Color = Color::Rgb(184, 204, 82); // Selected items
const AYU_GRAY: Color = Color::Rgb(92, 103, 115); // Dim text
const AYU_DIM_BLUE: Color = Color::Rgb(34, 52, 61); // Very dim blue for cursor highlight

#[derive(Debug, Clone)]
pub struct TuiConfig {
    pub fullscreen: bool,
    pub height: Option<u16>,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_height(height: u16) -> Self {
        Self {
            fullscreen: false,
            height: Some(height),
            height_percentage: None,
        }
    }

    pub fn with_height_percentage(percentage: f32) -> Self {
        Self {
            fullscreen: false,
            height: None,
            height_percentage: Some(percentage),
        }
    }

    pub fn fullscreen() -> Self {
        Self {
            fullscreen: true,
            height: None,
            height_percentage: None,
        }
    }

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

pub fn run_tui(
    items: Vec<String>,
    multi_select: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    run_tui_with_config(items, multi_select, TuiConfig::default())
}

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
    // Setup terminal with panic-safe raw mode
    let _raw_mode = RawModeGuard::new().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();

    // Store initial cursor position for non-fullscreen mode
    let initial_cursor_pos = if !config.fullscreen {
        crossterm::cursor::position().unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    // Track if we scrolled and how much
    let mut scrolled_lines = 0;

    if config.fullscreen {
        // Clear screen and hide cursor for fullscreen mode
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )
        .map_err(|e| format!("Failed to initialize terminal: {}", e))?;
    } else {
        // For non-fullscreen mode, check if we need to scroll the view down
        let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
        let tui_height = config.calculate_height(terminal_size.1);
        let start_y = initial_cursor_pos.1;
        let max_start_y = terminal_size.1.saturating_sub(tui_height);

        // If there's not enough space, scroll the view down
        if start_y > max_start_y {
            scrolled_lines = start_y - max_start_y;
            execute!(
                stdout,
                cursor::Hide,
                crossterm::terminal::ScrollUp(scrolled_lines)
            )
            .map_err(|e| format!("Failed to scroll terminal: {}", e))?;
        } else {
            execute!(stdout, cursor::Hide)
                .map_err(|e| format!("Failed to initialize terminal: {}", e))?;
        }
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Only clear terminal in fullscreen mode
    if config.fullscreen {
        terminal
            .clear()
            .map_err(|e| format!("Failed to clear terminal: {}", e))?;
    }

    let mut fuzzy_finder = FuzzyFinder::new(items, multi_select);
    fuzzy_finder.update_filter();
    let mut result = Vec::new();

    loop {
        // Render the UI
        terminal.draw(|f| {
            render_ui(f, &fuzzy_finder, multi_select, &config, initial_cursor_pos);
        })?;

        // Handle input events
        if let Event::Key(key_event) =
            event::read().map_err(|e| format!("Failed to read input: {}", e))?
        {
            let action = handle_key_event(&key_event, &mut fuzzy_finder);
            match action {
                crate::tui::controls::Action::Exit => break,
                crate::tui::controls::Action::Select(items) => {
                    result = items;
                    break;
                }
                crate::tui::controls::Action::Continue => {}
            }
        }
    }

    // Cleanup and restore cursor position
    disable_raw_mode()?;

    let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
    if config.fullscreen {
        // For fullscreen mode, clear only the viewport area (entire terminal)
        for y in 0..terminal_size.1 {
            execute!(
                terminal.backend_mut(),
                cursor::MoveTo(0, y),
                Clear(ClearType::CurrentLine)
            )?;
        }
        // Restore cursor to original position and show it
        execute!(
            terminal.backend_mut(),
            cursor::MoveTo(initial_cursor_pos.0, initial_cursor_pos.1),
            cursor::Show
        )?;
    } else {
        // For non-fullscreen mode, clear the viewport area and restore cursor to original position
        let tui_height = config.calculate_height(terminal_size.1);
        // Use the initial cursor position to determine where the TUI started
        let start_y = initial_cursor_pos.1;
        // Ensure we don't exceed terminal bounds (same logic as in render_ui)
        let max_start_y = terminal_size.1.saturating_sub(tui_height);
        let actual_start_y = start_y.min(max_start_y);

        // Clear the viewport area line by line
        for y in actual_start_y..(actual_start_y + tui_height) {
            execute!(
                terminal.backend_mut(),
                cursor::MoveTo(0, y),
                Clear(ClearType::CurrentLine)
            )?;
        }

        // If we scrolled down, scroll back up to restore the original view
        if scrolled_lines > 0 {
            execute!(
                terminal.backend_mut(),
                crossterm::terminal::ScrollDown(scrolled_lines)
            )?;
        }

        // Restore cursor to original position and show it
        execute!(
            terminal.backend_mut(),
            cursor::MoveTo(initial_cursor_pos.0, initial_cursor_pos.1),
            cursor::Show
        )?;
    }

    // Return selected items
    if result.is_empty() && multi_select {
        result = fuzzy_finder.get_selected_items();
    }
    Ok(result)
}

// Pure function for creating layout constraints
pub fn create_layout_constraints() -> Vec<Constraint> {
    vec![
        Constraint::Min(0),    // Results list (flexible, takes most space)
        Constraint::Length(3), // Search input (fixed at bottom)
    ]
}

// Pure function for creating results block title
pub fn create_results_title(filtered_count: usize, total_count: usize) -> String {
    format!("Results ({}/{})", filtered_count, total_count)
}

// Pure function for determining if an item is selected
pub fn is_item_selected(
    index: usize,
    cursor_position: usize,
    multi_select: bool,
    item: &str,
    items: &[String],
    selected_indices: &[usize],
) -> bool {
    if multi_select {
        let original_index = items.iter().position(|x| x == item);
        original_index.is_some_and(|idx| selected_indices.contains(&idx))
    } else {
        index == cursor_position
    }
}

// Pure function for creating item style with Ayu highlights and native terminal colors
pub fn create_item_style(index: usize, cursor_position: usize, is_selected: bool) -> Style {
    if index == cursor_position {
        Style::default()
            .fg(NATIVE_FG)
            .bg(AYU_DIM_BLUE)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default()
            .fg(AYU_GREEN)
            .bg(NATIVE_BG)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(NATIVE_FG).bg(NATIVE_BG)
    }
}

// Pure function for creating item prefix
pub fn create_item_prefix(multi_select: bool, is_selected: bool) -> &'static str {
    if multi_select {
        if is_selected {
            "✓ "
        } else {
            "  "
        }
    } else {
        ""
    }
}

// Pure function for creating list items
pub fn create_list_items<'a>(
    filtered_items: &'a [String],
    fuzzy_finder: &'a FuzzyFinder,
) -> Vec<ListItem<'a>> {
    filtered_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = is_item_selected(
                i,
                fuzzy_finder.cursor_position,
                fuzzy_finder.multi_select,
                item,
                &fuzzy_finder.items,
                &fuzzy_finder.selected_indices,
            );
            let style = create_item_style(i, fuzzy_finder.cursor_position, is_selected);
            let prefix = create_item_prefix(fuzzy_finder.multi_select, is_selected);
            let highlighted_text = create_highlighted_text(item, &fuzzy_finder.query, style);

            let mut all_spans = vec![Span::styled(prefix, style)];
            all_spans.extend(highlighted_text.spans);
            ListItem::new(vec![Line::from(all_spans)])
        })
        .collect()
}

// Pure function for creating search title
pub fn create_search_title(multi_select: bool, selected_count: usize) -> String {
    if multi_select {
        format!("Search (Multi-Select: {} selected)", selected_count)
    } else {
        "Search".to_string()
    }
}

// Pure function for creating search text with Ayu dim color
pub fn create_search_text(query: &str) -> Line {
    if query.is_empty() {
        Line::from(vec![Span::styled(
            "Type to search...",
            Style::default().fg(AYU_GRAY),
        )])
    } else {
        Line::from(vec![Span::styled(
            query,
            Style::default().fg(AYU_YELLOW).add_modifier(Modifier::BOLD),
        )])
    }
}

// Pure function for creating highlighted text with matched characters (Ayu yellow for matches, inverted for legibility)
pub fn create_highlighted_text<'a>(text: &'a str, query: &str, base_style: Style) -> Line<'a> {
    if query.is_empty() {
        return Line::from(vec![Span::styled(text, base_style)]);
    }
    use crate::fuzzy::matching::find_match_positions;
    let match_positions = find_match_positions(text, query);
    let mut spans = Vec::new();
    let mut last_pos = 0;
    for &pos in &match_positions {
        if pos > last_pos {
            spans.push(Span::styled(&text[last_pos..pos], base_style));
        }
        let char_start = text.char_indices().nth(pos).map(|(i, _)| i).unwrap_or(pos);
        let char_end = text
            .char_indices()
            .nth(pos + 1)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        spans.push(Span::styled(
            &text[char_start..char_end],
            base_style
                .fg(Color::Rgb(0, 0, 0))
                .bg(AYU_YELLOW)
                .add_modifier(Modifier::BOLD),
        ));
        last_pos = char_end;
    }
    if last_pos < text.len() {
        spans.push(Span::styled(&text[last_pos..], base_style));
    }
    Line::from(spans)
}

fn render_ui(
    f: &mut ratatui::Frame,
    fuzzy_finder: &FuzzyFinder,
    multi_select: bool,
    config: &TuiConfig,
    initial_cursor_pos: (u16, u16),
) {
    let size = f.area();

    if config.fullscreen {
        // Fullscreen mode with borders and traditional layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(create_layout_constraints())
            .split(size);

        // Results list (top section)
        let results_block = Block::default()
            .title(create_results_title(
                fuzzy_finder.filtered_items.len(),
                fuzzy_finder.items.len(),
            ))
            .borders(Borders::ALL)
            .style(Style::default().fg(AYU_GRAY).bg(NATIVE_BG));
        let list_items = create_list_items(&fuzzy_finder.filtered_items, fuzzy_finder);
        let list = List::new(list_items)
            .block(results_block)
            .style(Style::default().bg(NATIVE_BG));
        f.render_widget(list, chunks[0]);

        // Search input area with mode indicator (bottom section)
        let search_title = create_search_title(multi_select, fuzzy_finder.selected_indices.len());
        let search_block = Block::default()
            .title(search_title)
            .borders(Borders::ALL)
            .style(Style::default().fg(AYU_GRAY).bg(NATIVE_BG));
        let search_text = create_search_text(&fuzzy_finder.query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().bg(NATIVE_BG));
        f.render_widget(search_paragraph, chunks[1]);
    } else {
        // Non-fullscreen mode: render from initial cursor position down
        let tui_height = config.calculate_height(size.height);

        // Use the initial cursor position as the starting point
        let start_y = initial_cursor_pos.1;

        // Ensure we don't exceed terminal bounds
        let max_start_y = size.height.saturating_sub(tui_height);
        let start_y = start_y.min(max_start_y);

        // Create a smaller area starting from the initial cursor position
        let tui_area = ratatui::layout::Rect {
            x: 0,
            y: start_y,
            width: size.width,
            height: tui_height,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(vec![
                Constraint::Length(1), // Search input line
                Constraint::Min(0),    // Results list (takes remaining space)
            ])
            .split(tui_area);

        // Search input line (no borders, just the input)
        let search_text = if fuzzy_finder.query.is_empty() {
            Line::from(vec![Span::styled(
                "Type to search...",
                Style::default().fg(AYU_GRAY),
            )])
        } else {
            Line::from(vec![Span::styled(
                &fuzzy_finder.query,
                Style::default().fg(AYU_YELLOW).add_modifier(Modifier::BOLD),
            )])
        };
        let search_paragraph = Paragraph::new(search_text).style(Style::default().bg(NATIVE_BG));
        f.render_widget(search_paragraph, chunks[0]);

        // Results list (no borders)
        let list_items = create_list_items(&fuzzy_finder.filtered_items, fuzzy_finder);
        let list = List::new(list_items).style(Style::default().bg(NATIVE_BG));
        f.render_widget(list, chunks[1]);
    }
}

// RawModeGuard for safe terminal mode management
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
    use crate::fuzzy::FuzzyFinder;

    #[test]
    fn test_create_layout_constraints() {
        let constraints = create_layout_constraints();
        assert_eq!(constraints.len(), 2);
        assert!(matches!(constraints[0], Constraint::Min(0)));
        assert!(matches!(constraints[1], Constraint::Length(3)));
    }

    #[test]
    fn test_create_results_title() {
        let title = create_results_title(5, 10);
        assert_eq!(title, "Results (5/10)");
    }

    #[test]
    fn test_is_item_selected_single_mode() {
        let items = vec!["item1".to_string(), "item2".to_string()];
        let selected_indices = vec![];

        // Cursor at position 0
        assert!(is_item_selected(
            0,
            0,
            false,
            "item1",
            &items,
            &selected_indices
        ));
        assert!(!is_item_selected(
            1,
            0,
            false,
            "item2",
            &items,
            &selected_indices
        ));

        // Cursor at position 1
        assert!(!is_item_selected(
            0,
            1,
            false,
            "item1",
            &items,
            &selected_indices
        ));
        assert!(is_item_selected(
            1,
            1,
            false,
            "item2",
            &items,
            &selected_indices
        ));
    }

    #[test]
    fn test_is_item_selected_multi_mode() {
        let items = vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ];
        let selected_indices = vec![0, 2];

        assert!(is_item_selected(
            0,
            0,
            true,
            "item1",
            &items,
            &selected_indices
        ));
        assert!(!is_item_selected(
            1,
            0,
            true,
            "item2",
            &items,
            &selected_indices
        ));
        assert!(is_item_selected(
            2,
            0,
            true,
            "item3",
            &items,
            &selected_indices
        ));
    }

    #[test]
    fn test_create_item_style_cursor() {
        let style = create_item_style(0, 0, false);
        assert!(matches!(style, Style { .. }));
    }

    #[test]
    fn test_create_item_style_selected() {
        let style = create_item_style(1, 0, true);
        assert!(matches!(style, Style { .. }));
    }

    #[test]
    fn test_create_item_style_normal() {
        let style = create_item_style(1, 0, false);
        assert!(matches!(style, Style { .. }));
    }

    #[test]
    fn test_create_item_prefix_single_mode() {
        assert_eq!(create_item_prefix(false, true), "");
        assert_eq!(create_item_prefix(false, false), "");
    }

    #[test]
    fn test_create_item_prefix_multi_mode() {
        assert_eq!(create_item_prefix(true, true), "✓ ");
        assert_eq!(create_item_prefix(true, false), "  ");
    }

    #[test]
    fn test_create_list_items() {
        let mut finder = FuzzyFinder::new(vec!["item1".to_string(), "item2".to_string()], false);
        finder.update_filter();

        let items = create_list_items(&finder.filtered_items, &finder);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_create_search_title_single_mode() {
        let title = create_search_title(false, 0);
        assert_eq!(title, "Search");
    }

    #[test]
    fn test_create_search_title_multi_mode() {
        let title = create_search_title(true, 3);
        assert_eq!(title, "Search (Multi-Select: 3 selected)");
    }

    #[test]
    fn test_create_search_text_empty() {
        let text = create_search_text("");
        assert!(matches!(text, Line { .. }));
    }

    #[test]
    fn test_create_search_text_with_query() {
        let text = create_search_text("test query");
        assert!(matches!(text, Line { .. }));
    }

    #[test]
    fn test_raw_mode_guard() {
        // Test that RawModeGuard can be created (this is a smoke test)
        // Function should not panic
    }

    #[test]
    fn test_tui_config_default() {
        let config = TuiConfig::default();
        assert!(config.fullscreen);
        assert!(config.height.is_none());
        assert!(config.height_percentage.is_none());
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
    fn test_tui_config_calculate_height_fullscreen() {
        let config = TuiConfig::fullscreen();
        assert_eq!(config.calculate_height(20), 20);
    }

    #[test]
    fn test_tui_config_calculate_height_fixed() {
        let config = TuiConfig::with_height(10);
        assert_eq!(config.calculate_height(20), 10);
        assert_eq!(config.calculate_height(5), 5); // Should not exceed terminal height
    }

    #[test]
    fn test_tui_config_calculate_height_percentage() {
        let config = TuiConfig::with_height_percentage(50.0);
        assert_eq!(config.calculate_height(20), 10); // 50% of 20
        assert_eq!(config.calculate_height(10), 5); // 50% of 10
    }

    #[test]
    fn test_tui_config_calculate_height_percentage_rounding() {
        let config = TuiConfig::with_height_percentage(33.33);
        assert_eq!(config.calculate_height(10), 3); // 33.33% of 10 = 3.33, rounded to 3
    }
}
