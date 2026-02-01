use crate::fuzzy::FuzzyFinder;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions that can be performed by the TUI
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Continue processing
    Continue,
    /// Exit the application
    Exit,
    /// Select items and exit
    Select(Vec<String>),
}

/// Handle key events and return appropriate actions
pub fn handle_key_event(key_event: &KeyEvent, fuzzy_finder: &mut FuzzyFinder) -> Action {
    match key_event.code {
        KeyCode::Char(c) => {
            if (c == 'q' || c == 'c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                Action::Exit
            } else if c == ' ' && fuzzy_finder.is_multi_select() {
                fuzzy_finder.toggle_selection();
                Action::Continue
            } else {
                // For synchronous version, we can't update the query asynchronously
                // This is handled differently in the async version
                Action::Continue
            }
        }
        KeyCode::Backspace => {
            // For synchronous version, we can't update the query asynchronously
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
                // Move to next item without wrapping (stop at bottom)
                fuzzy_finder.move_cursor_clamped(1);
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
            } else {
                Action::Continue
            }
        }
        KeyCode::Esc => Action::Exit,
        _ => Action::Continue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuzzy::FuzzyFinder;

    #[test]
    fn test_action_enum_variants() {
        let continue_action = Action::Continue;
        let exit_action = Action::Exit;
        let select_action = Action::Select(vec!["test".to_string()]);

        assert_ne!(continue_action, exit_action);
        assert_ne!(continue_action, select_action);
        assert_ne!(exit_action, select_action);
    }

    #[tokio::test]
    async fn test_handle_key_event_char_input() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        // In synchronous mode, char input doesn't update the query
        assert_eq!(action, Action::Continue);
    }

    #[tokio::test]
    async fn test_handle_key_event_ctrl_q() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Exit);
    }

    #[tokio::test]
    async fn test_handle_key_event_ctrl_c() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Exit);
    }

    #[tokio::test]
    async fn test_handle_key_event_space_multi_select() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, true).await;

        let key_event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
        // The selection should be toggled
        assert!(!finder.get_selected_items().is_empty());
    }

    #[tokio::test]
    async fn test_handle_key_event_space_single_select() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
        // In single select mode, space should not toggle selection
    }

    #[tokio::test]
    async fn test_handle_key_event_backspace() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        // In synchronous mode, backspace doesn't update the query
        assert_eq!(action, Action::Continue);
    }

    #[tokio::test]
    async fn test_handle_key_event_backspace_empty() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
    }

    #[tokio::test]
    async fn test_handle_key_event_up_arrow() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let initial_position = finder.get_cursor_position();
        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
        // Cursor should have moved up (wrapped to the end)
        assert_ne!(finder.get_cursor_position(), initial_position);
    }

    #[tokio::test]
    async fn test_handle_key_event_down_arrow() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let initial_position = finder.get_cursor_position();
        let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
        // Cursor should have moved down
        assert_ne!(finder.get_cursor_position(), initial_position);
    }

    #[tokio::test]
    async fn test_handle_key_event_tab_multi_select() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, true).await;

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
        // The selection should be toggled
        assert!(!finder.get_selected_items().is_empty());
    }

    #[tokio::test]
    async fn test_handle_key_event_tab_single_select() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
        // In single select mode, tab should not toggle selection
    }

    #[tokio::test]
    async fn test_handle_key_event_enter_single_select() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        match action {
            Action::Select(selected) => {
                assert_eq!(selected.len(), 1);
                assert_eq!(selected[0], "apple");
            }
            _ => panic!("Expected Select action"),
        }
    }

    #[tokio::test]
    async fn test_handle_key_event_enter_multi_select() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, true).await;

        // First toggle a selection
        let space_event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
        handle_key_event(&space_event, &mut finder);

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        match action {
            Action::Select(selected) => {
                assert_eq!(selected.len(), 1);
                assert_eq!(selected[0], "apple");
            }
            _ => panic!("Expected Select action"),
        }
    }

    #[tokio::test]
    async fn test_handle_key_event_enter_empty_results() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        // Set a query that doesn't match anything
        finder.set_query("xyz".to_string()).await;

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
    }

    #[tokio::test]
    async fn test_handle_key_event_escape() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Exit);
    }

    #[tokio::test]
    async fn test_handle_key_event_unknown() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::F(1), KeyModifiers::empty());
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
    }

    #[tokio::test]
    async fn test_handle_key_event_with_modifiers() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::SHIFT);
        let action = handle_key_event(&key_event, &mut finder);

        assert_eq!(action, Action::Continue);
    }
}
