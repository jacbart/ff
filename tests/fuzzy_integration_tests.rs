use ff::fuzzy::FuzzyFinder;

#[tokio::test]
async fn test_fuzzy_finder_new() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let finder = FuzzyFinder::with_items_async(items.clone(), false).await;

    assert_eq!(finder.get_filtered_items(), items.as_slice());
    assert_eq!(finder.get_query(), "");
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_fuzzy_finder_new_multi_select() {
    let items = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let finder = FuzzyFinder::with_items_async(items.clone(), true).await;

    assert_eq!(finder.get_filtered_items(), items.as_slice());
    assert_eq!(finder.get_query(), "");
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_fuzzy_finder_empty_items() {
    let finder = FuzzyFinder::with_items_async(vec![], false).await;

    assert!(finder.get_filtered_items().is_empty());
    assert_eq!(finder.get_query(), "");
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_update_filter_empty_query() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.set_query("".to_string()).await;

    assert_eq!(
        finder.get_filtered_items(),
        vec!["apple".to_string(), "banana".to_string()].as_slice()
    );
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_update_filter_with_query() {
    let mut finder = FuzzyFinder::with_items_async(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    )
    .await;
    finder.set_query("ap".to_string()).await;

    assert_eq!(
        finder.get_filtered_items(),
        vec!["apple".to_string()].as_slice()
    );
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_update_filter_no_matches() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.set_query("xyz".to_string()).await;

    assert!(finder.get_filtered_items().is_empty());
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_update_filter_case_insensitive() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["Apple".to_string(), "Banana".to_string()], false).await;
    finder.set_query("ap".to_string()).await;

    assert_eq!(
        finder.get_filtered_items(),
        vec!["Apple".to_string()].as_slice()
    );
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_move_cursor_up() {
    let mut finder = FuzzyFinder::with_items_async(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    )
    .await;

    // Move cursor to position 1
    finder.move_cursor(1);
    assert_eq!(finder.get_cursor_position(), 1);

    // Move cursor up
    finder.move_cursor(-1);
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_move_cursor_up_at_top() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.move_cursor(-1);

    assert_eq!(finder.get_cursor_position(), 1); // Should wrap to bottom
}

#[tokio::test]
async fn test_move_cursor_down() {
    let mut finder = FuzzyFinder::with_items_async(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    )
    .await;
    finder.move_cursor(1);

    assert_eq!(finder.get_cursor_position(), 1);
}

#[tokio::test]
async fn test_move_cursor_down_at_bottom() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.move_cursor(1);
    finder.move_cursor(1);

    assert_eq!(finder.get_cursor_position(), 0); // Should wrap to top
}

#[tokio::test]
async fn test_move_cursor_with_empty_filtered_items() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.set_query("xyz".to_string()).await;
    finder.move_cursor(1);

    assert_eq!(finder.get_cursor_position(), 0); // Should stay at 0 when no items
}

#[tokio::test]
async fn test_toggle_selection_single_mode() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.toggle_selection();

    assert_eq!(finder.get_selected_items(), vec!["apple".to_string()]);
}

#[tokio::test]
async fn test_toggle_selection_multi_mode() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], true).await;
    finder.toggle_selection();
    finder.move_cursor(1);
    finder.toggle_selection();

    let mut selected = finder.get_selected_items();
    selected.sort();
    assert_eq!(selected, vec!["apple".to_string(), "banana".to_string()]);
}

#[tokio::test]
async fn test_toggle_selection_remove() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], true).await;
    finder.toggle_selection();
    finder.toggle_selection(); // Toggle again to remove

    assert!(finder.get_selected_items().is_empty());
}

#[tokio::test]
async fn test_get_selected_items_single_mode() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.toggle_selection();

    let selected = finder.get_selected_items();
    assert_eq!(selected, vec!["apple".to_string()]);
}

#[tokio::test]
async fn test_get_selected_items_multi_mode() {
    let mut finder = FuzzyFinder::with_items_async(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        true,
    )
    .await;
    finder.toggle_selection();
    finder.move_cursor(1);
    finder.toggle_selection();
    finder.move_cursor(1);
    finder.toggle_selection();

    let mut selected = finder.get_selected_items();
    selected.sort();
    assert_eq!(
        selected,
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string()
        ]
    );
}

#[tokio::test]
async fn test_get_selected_items_empty() {
    let finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    let selected = finder.get_selected_items();
    assert!(selected.is_empty());
}

#[tokio::test]
async fn test_query_caching() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;

    // First query
    finder.set_query("ap".to_string()).await;
    let first_result = finder.get_filtered_items().to_vec();

    // Second query (should use cache)
    finder.set_query("ap".to_string()).await;
    let second_result = finder.get_filtered_items().to_vec();

    assert_eq!(first_result, second_result);
    assert_eq!(first_result, vec!["apple".to_string()]);
}

#[tokio::test]
async fn test_cursor_position_reset() {
    let mut finder = FuzzyFinder::with_items_async(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    )
    .await;

    finder.move_cursor(2);
    assert_eq!(finder.get_cursor_position(), 2);

    finder.set_query("ap".to_string()).await;
    assert_eq!(finder.get_cursor_position(), 0); // Should reset to 0
}

#[tokio::test]
async fn test_cursor_position_empty_results() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.move_cursor(1);
    finder.set_query("xyz".to_string()).await;

    assert_eq!(finder.get_cursor_position(), 0); // Should reset to 0 when no results
}

#[tokio::test]
async fn test_large_dataset_parallel_filtering() {
    let items: Vec<String> = (0..1000).map(|i| format!("item_{}", i)).collect();
    let mut finder = FuzzyFinder::with_items_async(items, false).await;

    finder.set_query("item_5".to_string()).await;
    let filtered = finder.get_filtered_items();

    assert!(!filtered.is_empty());
    // Check that all filtered items match the fuzzy pattern "item_5"
    // This means they should contain the characters 'i', 't', 'e', 'm', '_', '5' in sequence
    assert!(filtered.iter().all(|item| {
        let item_lower = item.to_lowercase();
        let query = "item_5";
        let mut query_chars = query.chars().peekable();
        let mut item_chars = item_lower.chars();

        while let Some(query_char) = query_chars.peek() {
            if let Some(item_char) = item_chars.next() {
                if item_char == *query_char {
                    query_chars.next();
                }
            } else {
                return false;
            }
        }
        query_chars.peek().is_none()
    }));
}

#[tokio::test]
async fn test_special_characters_in_query() {
    let mut finder = FuzzyFinder::with_items_async(
        vec!["test-item".to_string(), "normal_item".to_string()],
        false,
    )
    .await;

    finder.set_query("test-item".to_string()).await;
    assert_eq!(
        finder.get_filtered_items(),
        vec!["test-item".to_string()].as_slice()
    );
}

#[tokio::test]
async fn test_single_item_list() {
    let mut finder = FuzzyFinder::with_items_async(vec!["single".to_string()], false).await;
    finder.set_query("sin".to_string()).await;

    assert_eq!(
        finder.get_filtered_items(),
        vec!["single".to_string()].as_slice()
    );
    assert_eq!(finder.get_cursor_position(), 0);
}

#[tokio::test]
async fn test_edge_case_empty_query_after_non_empty() {
    let mut finder = FuzzyFinder::with_items_async(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    )
    .await;

    finder.set_query("ap".to_string()).await;
    assert_eq!(finder.get_filtered_items().len(), 1);

    finder.set_query("".to_string()).await;
    assert_eq!(finder.get_filtered_items().len(), 3);
}

#[tokio::test]
async fn test_cursor_boundary_conditions() {
    let mut finder = FuzzyFinder::with_items_async(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        false,
    )
    .await;

    // Test moving cursor beyond boundaries
    // With 3 items, moving by 10 positions: 10 % 3 = 1, so position 1
    finder.move_cursor(10);
    assert_eq!(finder.get_cursor_position(), 1); // Should wrap around correctly

    // Moving by -10 from position 1: 1 + (-10) = -9, abs(-9) % 3 = 0, so position 0
    finder.move_cursor(-10);
    assert_eq!(finder.get_cursor_position(), 0); // Should wrap around correctly
}

#[tokio::test]
async fn test_multi_select_complex_toggles() {
    let mut finder = FuzzyFinder::with_items_async(
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
        true,
    )
    .await;

    // Select first item
    finder.toggle_selection();
    assert_eq!(finder.get_selected_items().len(), 1);

    // Move to second item and select
    finder.move_cursor(1);
    finder.toggle_selection();
    assert_eq!(finder.get_selected_items().len(), 2);

    // Move back to first item and deselect
    finder.move_cursor(-1);
    finder.toggle_selection();
    assert_eq!(finder.get_selected_items().len(), 1);
}

#[tokio::test]
async fn test_multi_select_repeated_toggles() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], true).await;

    // Toggle same item multiple times
    finder.toggle_selection();
    finder.toggle_selection();
    finder.toggle_selection();

    let selected = finder.get_selected_items();
    assert_eq!(selected.len(), 1); // Should be selected after odd number of toggles
}

#[tokio::test]
async fn test_match_positions() {
    let mut finder =
        FuzzyFinder::with_items_async(vec!["apple".to_string(), "banana".to_string()], false).await;
    finder.set_query("ap".to_string()).await;

    // Check that match positions are calculated
    let positions = finder.get_match_positions(0);
    assert!(positions.is_some());
    if let Some(match_pos) = positions {
        assert!(!match_pos.positions.is_empty());
        assert!(match_pos.score > 0.0);
    }
}
