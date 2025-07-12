use ff::fuzzy::{fuzzy_match, FuzzyFinder};

#[test]
fn test_fuzzy_finder_new() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let finder = FuzzyFinder::new(items.clone(), false);

    assert_eq!(finder.items, items);
    assert_eq!(finder.query, "");
    assert_eq!(finder.filtered_items, items);
    assert_eq!(finder.cursor_position, 0);
    assert!(!finder.multi_select);
    assert!(finder.selected_indices.is_empty());
}

#[test]
fn test_fuzzy_finder_new_multi_select() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let finder = FuzzyFinder::new(items.clone(), true);

    assert_eq!(finder.items, items);
    assert_eq!(finder.query, "");
    assert_eq!(finder.filtered_items, items);
    assert_eq!(finder.cursor_position, 0);
    assert!(finder.multi_select);
    assert!(finder.selected_indices.is_empty());
}

#[test]
fn test_fuzzy_finder_empty_items() {
    let finder = FuzzyFinder::new(vec![], false);

    assert!(finder.items.is_empty());
    assert_eq!(finder.query, "");
    assert!(finder.filtered_items.is_empty());
    assert_eq!(finder.cursor_position, 0);
    assert!(!finder.multi_select);
    assert!(finder.selected_indices.is_empty());
}

#[test]
fn test_fuzzy_match_direct() {
    let result = fuzzy_match("apple", "apple");
    assert!(result);
}

#[test]
fn test_fuzzy_match_case_insensitive() {
    let result = fuzzy_match(&"Apple".to_lowercase(), &"apple".to_lowercase());
    assert!(result);
}

#[test]
fn test_fuzzy_match_substring() {
    let result = fuzzy_match("apple", "app");
    assert!(result);
}

#[test]
fn test_fuzzy_match_character_sequence() {
    let result = fuzzy_match("apple", "apl");
    assert!(result);
}

#[test]
fn test_fuzzy_match_no_match() {
    let result = fuzzy_match("apple", "xyz");
    assert!(!result);
}

#[test]
fn test_fuzzy_match_empty_query() {
    let result = fuzzy_match("apple", "");
    assert!(result);
}

#[test]
fn test_fuzzy_match_empty_item() {
    let result = fuzzy_match("", "apple");
    assert!(!result);
}

#[test]
fn test_fuzzy_match_both_empty() {
    let result = fuzzy_match("", "");
    assert!(result);
}

#[test]
fn test_update_filter_empty_query() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
    finder.query = "".to_string();
    finder.update_filter();

    assert_eq!(
        finder.filtered_items,
        vec!["apple".to_string(), "banana".to_string()]
    );
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_update_filter_with_query() {
    let mut finder = FuzzyFinder::new(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    );
    finder.query = "ap".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_update_filter_no_matches() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
    finder.query = "xyz".to_string();
    finder.update_filter();

    assert!(finder.filtered_items.is_empty());
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_update_filter_case_insensitive() {
    let mut finder = FuzzyFinder::new(vec!["Apple".to_string(), "Banana".to_string()], false);
    finder.query = "ap".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, vec!["Apple".to_string()]);
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_move_cursor_up() {
    let mut finder = FuzzyFinder::new(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    );
    finder.cursor_position = 1;
    finder.move_cursor(-1);

    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_move_cursor_up_at_top() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
    finder.cursor_position = 0;
    finder.move_cursor(-1);

    assert_eq!(finder.cursor_position, 1); // Should wrap to bottom
}

#[test]
fn test_move_cursor_down() {
    let mut finder = FuzzyFinder::new(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    );
    finder.cursor_position = 0;
    finder.move_cursor(1);

    assert_eq!(finder.cursor_position, 1);
}

#[test]
fn test_move_cursor_down_at_bottom() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
    finder.cursor_position = 1;
    finder.move_cursor(1);

    assert_eq!(finder.cursor_position, 0); // Should wrap to top
}

#[test]
fn test_move_cursor_with_empty_filtered_items() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
    finder.query = "xyz".to_string();
    finder.update_filter();
    finder.cursor_position = 0;
    finder.move_cursor(1);

    assert_eq!(finder.cursor_position, 0); // Should stay at 0 when no items
}

#[test]
fn test_toggle_selection_single_mode() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
    finder.cursor_position = 0;
    finder.toggle_selection();

    assert_eq!(finder.selected_indices, vec![0]);
}

#[test]
fn test_toggle_selection_multi_mode() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], true);
    finder.cursor_position = 0;
    finder.toggle_selection();
    finder.cursor_position = 1;
    finder.toggle_selection();

    assert_eq!(finder.selected_indices, vec![0, 1]);
}

#[test]
fn test_toggle_selection_remove() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], true);
    finder.cursor_position = 0;
    finder.toggle_selection();
    finder.toggle_selection(); // Toggle again to remove

    assert!(finder.selected_indices.is_empty());
}

#[test]
fn test_get_selected_items_single_mode() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
    finder.cursor_position = 0;
    finder.toggle_selection();

    let selected = finder.get_selected_items();
    assert_eq!(selected, vec!["apple".to_string()]);
}

#[test]
fn test_get_selected_items_multi_mode() {
    let mut finder = FuzzyFinder::new(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        true,
    );
    finder.cursor_position = 0;
    finder.toggle_selection();
    finder.cursor_position = 2;
    finder.toggle_selection();

    let selected = finder.get_selected_items();
    assert_eq!(selected, vec!["apple".to_string(), "cherry".to_string()]);
}

#[test]
fn test_get_selected_items_empty() {
    let finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], true);

    let selected = finder.get_selected_items();
    assert!(selected.is_empty());
}

#[test]
fn test_query_caching() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);

    // First query
    finder.query = "ap".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items, vec!["apple".to_string()]);

    // Same query again - should use cache
    finder.update_filter();
    assert_eq!(finder.filtered_items, vec!["apple".to_string()]);

    // Different query
    finder.query = "ba".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items, vec!["banana".to_string()]);
}

#[test]
fn test_cursor_position_reset() {
    let mut finder = FuzzyFinder::new(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    );
    finder.cursor_position = 2;

    // Update filter should reset cursor position
    finder.query = "ap".to_string();
    finder.update_filter();
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_cursor_position_empty_results() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
    finder.cursor_position = 1;

    // Update filter with no matches
    finder.query = "xyz".to_string();
    finder.update_filter();
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_large_dataset_parallel_filtering() {
    let items: Vec<String> = (0..1000).map(|i| format!("item_{}", i)).collect();
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "item_5".to_string();
    finder.update_filter();

    // Should find items that match "item_5" in fuzzy matching
    assert!(!finder.filtered_items.is_empty());
    // Check that we get some results (the exact number depends on fuzzy matching logic)
    assert!(!finder.filtered_items.is_empty());
}

#[test]
fn test_special_characters_in_query() {
    let items = vec![
        "test-item".to_string(),
        "test_item".to_string(),
        "testitem".to_string(),
    ];
    let mut finder = FuzzyFinder::new(items, false);

    finder.query = "test-item".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, vec!["test-item".to_string()]);
}

#[test]
fn test_single_item_list() {
    let mut finder = FuzzyFinder::new(vec!["single".to_string()], false);

    finder.query = "sin".to_string();
    finder.update_filter();

    assert_eq!(finder.filtered_items, vec!["single".to_string()]);
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_edge_case_empty_query_after_non_empty() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);

    // First with a query
    finder.query = "ap".to_string();
    finder.update_filter();
    assert_eq!(finder.filtered_items, vec!["apple".to_string()]);

    // Then with empty query
    finder.query = "".to_string();
    finder.update_filter();
    assert_eq!(
        finder.filtered_items,
        vec!["apple".to_string(), "banana".to_string()]
    );
}

#[test]
fn test_cursor_boundary_conditions() {
    let mut finder = FuzzyFinder::new(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    );

    // Test cursor at boundaries
    finder.cursor_position = 0;
    finder.move_cursor(-1); // Should wrap to bottom
    assert_eq!(finder.cursor_position, 2);

    finder.cursor_position = 2;
    finder.move_cursor(1); // Should wrap to top
    assert_eq!(finder.cursor_position, 0);
}

#[test]
fn test_multi_select_complex_toggles() {
    let mut finder = FuzzyFinder::new(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        true,
    );

    // Select first and third items
    finder.cursor_position = 0;
    finder.toggle_selection();
    finder.cursor_position = 2;
    finder.toggle_selection();

    let selected = finder.get_selected_items();
    assert_eq!(selected, vec!["apple".to_string(), "cherry".to_string()]);

    // Deselect first item
    finder.cursor_position = 0;
    finder.toggle_selection();

    let selected = finder.get_selected_items();
    assert_eq!(selected, vec!["cherry".to_string()]);
}

#[test]
fn test_multi_select_repeated_toggles() {
    let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], true);

    // Toggle same item multiple times
    finder.cursor_position = 0;
    finder.toggle_selection(); // Select
    finder.toggle_selection(); // Deselect
    finder.toggle_selection(); // Select again

    let selected = finder.get_selected_items();
    assert_eq!(selected, vec!["apple".to_string()]);
}
