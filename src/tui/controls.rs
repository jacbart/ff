use crate::fuzzy::FuzzyFinder;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub enum Action {
    Continue,
    Exit,
    Select(Vec<String>),
}

pub fn handle_key_event(key_event: &KeyEvent, fuzzy_finder: &mut FuzzyFinder) -> Action {
    match key_event.code {
        KeyCode::Char(c) => {
            if c == 'q' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                Action::Exit
            } else if c == ' '
                && fuzzy_finder.multi_select
                && !fuzzy_finder.filtered_items.is_empty()
            {
                // Space bar toggles selection in multi-select mode
                fuzzy_finder.toggle_selection();
                Action::Continue
            } else {
                fuzzy_finder.query.push(c);
                fuzzy_finder.update_filter();
                Action::Continue
            }
        }
        KeyCode::Backspace => {
            fuzzy_finder.query.pop();
            fuzzy_finder.update_filter();
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
            // TAB toggles selection in multi-select mode
            if fuzzy_finder.multi_select && !fuzzy_finder.filtered_items.is_empty() {
                fuzzy_finder.toggle_selection();
            }
            Action::Continue
        }
        KeyCode::Enter => {
            if !fuzzy_finder.filtered_items.is_empty() {
                if fuzzy_finder.multi_select {
                    // In multi-select mode, Enter returns all selected items
                    let selected_items = fuzzy_finder.get_selected_items();
                    Action::Select(selected_items)
                } else {
                    // Single select - get the selected item and exit
                    let selected_item = &fuzzy_finder.filtered_items[fuzzy_finder.cursor_position];
                    if let Some(original_index) = fuzzy_finder
                        .items
                        .iter()
                        .position(|item| item == selected_item)
                    {
                        let selected_items = vec![fuzzy_finder.items[original_index].clone()];
                        Action::Select(selected_items)
                    } else {
                        Action::Exit
                    }
                }
            } else {
                Action::Exit
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
        let select_action = Action::Select(vec!["item".to_string()]);

        assert!(matches!(continue_action, Action::Continue));
        assert!(matches!(exit_action, Action::Exit));
        assert!(matches!(select_action, Action::Select(_)));
    }

    #[test]
    fn test_handle_key_event_char_input() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('a'),
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
        assert_eq!(finder.query, "a");
    }

    #[test]
    fn test_handle_key_event_ctrl_q() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('q'),
            crossterm::event::KeyModifiers::CONTROL,
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Exit));
    }

    #[test]
    fn test_handle_key_event_space_multi_select() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], true);
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(' '),
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
    }

    #[test]
    fn test_handle_key_event_space_single_select() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(' '),
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
        assert_eq!(finder.query, " ");
    }

    #[test]
    fn test_handle_key_event_backspace() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        finder.query = "abc".to_string();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Backspace,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
        assert_eq!(finder.query, "ab");
    }

    #[test]
    fn test_handle_key_event_backspace_empty() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        finder.query = "".to_string();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Backspace,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
        assert_eq!(finder.query, "");
    }

    #[test]
    fn test_handle_key_event_up_arrow() {
        let mut finder = FuzzyFinder::new(vec!["item1".to_string(), "item2".to_string()], false);
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Up,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
    }

    #[test]
    fn test_handle_key_event_down_arrow() {
        let mut finder = FuzzyFinder::new(vec!["item1".to_string(), "item2".to_string()], false);
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
    }

    #[test]
    fn test_handle_key_event_tab_multi_select() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], true);
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
    }

    #[test]
    fn test_handle_key_event_tab_single_select() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
    }

    #[test]
    fn test_handle_key_event_enter_single_select() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Select(_)));
    }

    #[test]
    fn test_handle_key_event_enter_multi_select() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], true);
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Select(_)));
    }

    #[test]
    fn test_handle_key_event_enter_empty_results() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        finder.query = "xyz".to_string();
        finder.update_filter();
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Exit));
    }

    #[test]
    fn test_handle_key_event_escape() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Exit));
    }

    #[test]
    fn test_handle_key_event_unknown() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(1),
            crossterm::event::KeyModifiers::empty(),
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
    }

    #[test]
    fn test_handle_key_event_with_modifiers() {
        let mut finder = FuzzyFinder::new(vec!["test".to_string()], false);
        let key_event = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('a'),
            crossterm::event::KeyModifiers::SHIFT,
        );

        let action = handle_key_event(&key_event, &mut finder);
        assert!(matches!(action, Action::Continue));
        assert_eq!(finder.query, "a"); // The key event handling doesn't auto-convert case
    }
}
