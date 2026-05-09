use crate::fuzzy::FuzzyFinder;
use crate::tui::controls::Action;
use crate::tui::preview::PreviewState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle key events in async mode
pub async fn handle_async_key_event(
    key_event: &KeyEvent,
    fuzzy_finder: &mut FuzzyFinder,
    preview_state: &mut PreviewState,
) -> Action {
    // Preview-focused navigation
    if preview_state.focused {
        match key_event.code {
            KeyCode::Up => {
                preview_state.scroll_up(1);
                return Action::Continue;
            }
            KeyCode::Down => {
                let max = preview_state.lines.len();
                preview_state.scroll_down(1, max);
                return Action::Continue;
            }
            KeyCode::Left => {
                preview_state.focused = false;
                return Action::Continue;
            }
            KeyCode::Esc => {
                preview_state.focused = false;
                return Action::Continue;
            }
            KeyCode::Enter => {
                let selected = fuzzy_finder.get_selected_items();
                if !selected.is_empty() {
                    return Action::Select(selected);
                } else if !fuzzy_finder.get_filtered_items().is_empty() {
                    let cursor_pos = fuzzy_finder.get_cursor_position();
                    let current_item = &fuzzy_finder.get_filtered_items()[cursor_pos];
                    let current_idx = fuzzy_finder.get_original_index(cursor_pos).unwrap();
                    return Action::Select(vec![(current_idx, current_item.clone())]);
                }
                return Action::Continue;
            }
            _ => {
                // Fall through to list handling
            }
        }
    }

    match key_event.code {
        KeyCode::Char(c) => {
            if (c == 'q' || c == 'c') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                Action::Exit
            } else if c == ' ' && fuzzy_finder.is_multi_select() {
                fuzzy_finder.toggle_selection();
                Action::Continue
            } else if c == 'p' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                preview_state.toggle_visible();
                Action::Continue
            } else if c == 'u' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                if preview_state.visible {
                    preview_state.scroll_up(available_height_for_preview(preview_state) / 2);
                }
                Action::Continue
            } else if c == 'd' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                if preview_state.visible {
                    let h = available_height_for_preview(preview_state);
                    preview_state.scroll_down(h / 2, preview_state.lines.len());
                }
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
        KeyCode::Left => {
            if preview_state.visible {
                preview_state.focused = false;
            }
            Action::Continue
        }
        KeyCode::Right => {
            if preview_state.visible {
                preview_state.focused = true;
            }
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
                let cursor_pos = fuzzy_finder.get_cursor_position();
                let current_item = &fuzzy_finder.get_filtered_items()[cursor_pos];
                let current_idx = fuzzy_finder.get_original_index(cursor_pos).unwrap();
                Action::Select(vec![(current_idx, current_item.clone())])
            } else if fuzzy_finder.is_multi_select()
                && !fuzzy_finder.get_filtered_items().is_empty()
            {
                // In multi-select mode, if no items are selected, select the current item
                let cursor_pos = fuzzy_finder.get_cursor_position();
                let current_item = &fuzzy_finder.get_filtered_items()[cursor_pos];
                let current_idx = fuzzy_finder.get_original_index(cursor_pos).unwrap();
                Action::Select(vec![(current_idx, current_item.clone())])
            } else {
                Action::Continue
            }
        }
        KeyCode::Esc => {
            // Two-stage escape: first clears query, second exits
            if fuzzy_finder.get_query().is_empty() {
                Action::Exit
            } else {
                fuzzy_finder.set_query(String::new()).await;
                Action::Continue
            }
        }
        _ => Action::Continue,
    }
}

/// Helper for Ctrl+U/D scroll amount in preview pane
fn available_height_for_preview(preview_state: &PreviewState) -> usize {
    // Approximate: we don't have config here, use a reasonable default
    preview_state.lines.len().min(20)
}
