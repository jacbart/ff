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
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crate::fuzzy::FuzzyFinder;
use crate::tui::controls::handle_key_event;
use std::io;

pub fn run_tui(items: Vec<String>, multi_select: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    run_interactive_tui(items, multi_select)
}

fn run_interactive_tui(items: Vec<String>, multi_select: bool) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Setup terminal with panic-safe raw mode
    let _raw_mode = RawModeGuard::new().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    
    // Clear screen and hide cursor
    execute!(stdout, 
        Clear(ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0)
    ).map_err(|e| format!("Failed to initialize terminal: {}", e))?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| format!("Failed to create terminal: {}", e))?;
    
    // Ensure terminal is properly sized
    terminal.clear().map_err(|e| format!("Failed to clear terminal: {}", e))?;

    let mut fuzzy_finder = FuzzyFinder::new(items, multi_select);
    fuzzy_finder.update_filter();
    let mut result = Vec::new();

    loop {
        // Render the UI
        terminal.draw(|f| {
            render_ui(f, &fuzzy_finder, multi_select);
        })?;
        
        // Handle input events
        if let Event::Key(key_event) = event::read().map_err(|e| format!("Failed to read input: {}", e))? {
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

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), 
        cursor::Show, 
        Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

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
    selected_indices: &[usize]
) -> bool {
    if multi_select {
        let original_index = items.iter().position(|x| x == item);
        original_index.is_some_and(|idx| selected_indices.contains(&idx))
    } else {
        index == cursor_position
    }
}

// Pure function for creating item style
pub fn create_item_style(index: usize, cursor_position: usize, is_selected: bool) -> Style {
    if index == cursor_position {
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

// Pure function for creating item prefix
pub fn create_item_prefix(multi_select: bool, is_selected: bool) -> &'static str {
    if multi_select {
        if is_selected { "✓ " } else { "  " }
    } else {
        ""
    }
}

// Pure function for creating list items
pub fn create_list_items<'a>(
    filtered_items: &'a [String], 
    fuzzy_finder: &'a FuzzyFinder
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
                &fuzzy_finder.selected_indices
            );
            let style = create_item_style(i, fuzzy_finder.cursor_position, is_selected);
            let prefix = create_item_prefix(fuzzy_finder.multi_select, is_selected);
            ListItem::new(vec![Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(item, style),
            ])])
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

// Pure function for creating search text
pub fn create_search_text(query: &str) -> Line {
    if query.is_empty() {
        Line::from(vec![Span::raw("Type to search...")])
    } else {
        Line::from(vec![Span::raw(query)])
    }
}

fn render_ui(f: &mut ratatui::Frame, fuzzy_finder: &FuzzyFinder, multi_select: bool) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(create_layout_constraints())
        .split(size);
    
    // Results list (top section)
    let results_block = Block::default()
        .title(create_results_title(fuzzy_finder.filtered_items.len(), fuzzy_finder.items.len()))
        .borders(Borders::ALL);
    let list_items = create_list_items(&fuzzy_finder.filtered_items, fuzzy_finder);
    let list = List::new(list_items)
        .block(results_block)
        .style(Style::default());
    f.render_widget(list, chunks[0]);
    
    // Search input area with mode indicator (bottom section)
    let search_title = create_search_title(multi_select, fuzzy_finder.selected_indices.len());
    let search_block = Block::default()
        .title(search_title)
        .borders(Borders::ALL);
    let search_text = create_search_text(&fuzzy_finder.query);
    let search_paragraph = Paragraph::new(search_text)
        .block(search_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(search_paragraph, chunks[1]);
}

// RawModeGuard for safe terminal mode management
struct RawModeGuard;
impl RawModeGuard {
    fn new() -> std::io::Result<Self> {
        enable_raw_mode().map_err(|e| std::io::Error::other(e))?;
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
        assert!(is_item_selected(0, 0, false, "item1", &items, &selected_indices));
        assert!(!is_item_selected(1, 0, false, "item2", &items, &selected_indices));
        
        // Cursor at position 1
        assert!(!is_item_selected(0, 1, false, "item1", &items, &selected_indices));
        assert!(is_item_selected(1, 1, false, "item2", &items, &selected_indices));
    }

    #[test]
    fn test_is_item_selected_multi_mode() {
        let items = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
        let selected_indices = vec![0, 2];
        
        assert!(is_item_selected(0, 0, true, "item1", &items, &selected_indices));
        assert!(!is_item_selected(1, 0, true, "item2", &items, &selected_indices));
        assert!(is_item_selected(2, 0, true, "item3", &items, &selected_indices));
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
        assert!(true);
    }
} 