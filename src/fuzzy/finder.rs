use crate::fuzzy::scoring;
use crate::fuzzy::stream::ItemStream;

/// Match positions for highlighting
#[derive(Debug, Clone)]
pub struct MatchPositions {
    pub positions: Vec<usize>,
    pub score: i32,
}

/// Async fuzzy finder with streaming capabilities
pub struct FuzzyFinder {
    pub(crate) stream: ItemStream,
    pub(crate) query: String,
    pub(crate) filtered_items: Vec<String>,
    pub(crate) filtered_indices: Vec<usize>,
    pub(crate) match_positions: Vec<MatchPositions>,
    pub(crate) selected_items: std::collections::HashSet<usize>,
    pub(crate) cursor_position: usize,
    pub(crate) multi_select: bool,
    /// Cache stores (filtered_items, filtered_indices, match_positions) for each query
    pub(crate) query_cache:
        std::collections::HashMap<String, (Vec<String>, Vec<usize>, Vec<MatchPositions>)>,
}

impl FuzzyFinder {
    /// Create a new async fuzzy finder (empty)
    pub fn new(multi_select: bool) -> Self {
        let stream = ItemStream::new();
        Self {
            stream,
            query: String::new(),
            filtered_items: Vec::new(),
            filtered_indices: Vec::new(),
            match_positions: Vec::new(),
            selected_items: std::collections::HashSet::new(),
            cursor_position: 0,
            multi_select,
            query_cache: std::collections::HashMap::new(),
        }
    }

    /// Async constructor: create and add initial items
    pub async fn with_items_async(items: Vec<String>, multi_select: bool) -> Self {
        let mut finder = Self::new(multi_select);
        finder.add_items(items).await;
        finder
    }

    /// Update the filtered items based on the current query
    pub async fn update_filter(&mut self) {
        if self.query.is_empty() {
            let all_items = self.stream.get_all_items();
            self.filtered_items = Vec::new();
            self.filtered_indices = Vec::new();
            for (idx, item) in all_items.iter().enumerate() {
                if !item.is_empty() {
                    self.filtered_items.push(item.clone());
                    self.filtered_indices.push(idx);
                }
            }
            self.match_positions = self
                .filtered_items
                .iter()
                .map(|_| MatchPositions {
                    positions: Vec::new(),
                    score: 0,
                })
                .collect();
        } else if let Some(cached) = self.query_cache.get(&self.query) {
            self.filtered_items = cached.0.clone();
            self.filtered_indices = cached.1.clone();
            self.match_positions = cached.2.clone();
        } else {
            let all_items = self.stream.get_all_items();

            // Use the new scoring module for single-pass matching and scoring
            let scored_results = scoring::score_batch(&all_items, &self.query);

            // Extract filtered items and match positions (already sorted by score)
            self.filtered_items = scored_results
                .iter()
                .map(|(idx, _)| all_items[*idx].clone())
                .collect();

            self.filtered_indices = scored_results.iter().map(|(idx, _)| *idx).collect();

            self.match_positions = scored_results
                .into_iter()
                .map(|(_, result)| MatchPositions {
                    positions: result.positions,
                    score: result.score,
                })
                .collect();

            // Cache the results
            self.query_cache.insert(
                self.query.clone(),
                (
                    self.filtered_items.clone(),
                    self.filtered_indices.clone(),
                    self.match_positions.clone(),
                ),
            );
        }

        // Adjust cursor position
        if self.cursor_position >= self.filtered_items.len() {
            self.cursor_position = if self.filtered_items.is_empty() {
                0
            } else {
                self.filtered_items.len() - 1
            };
        }
    }

    /// Get match positions for a specific item index
    pub fn get_match_positions(&self, index: usize) -> Option<&MatchPositions> {
        self.match_positions.get(index)
    }

    /// Add new items asynchronously
    pub async fn add_items(&mut self, new_items: Vec<String>) {
        self.stream.add_items(new_items).await;
        // Clear cache when items change
        self.query_cache.clear();
        self.update_filter().await;
    }

    /// Move cursor up or down (wraps around)
    pub fn move_cursor(&mut self, direction: i32) {
        let len = self.filtered_items.len();
        if len == 0 {
            return;
        }

        // Handle large movements by using modulo arithmetic
        let current_pos = self.cursor_position as i32;
        let new_position = current_pos + direction;

        // Use modulo to handle wrapping correctly for both positive and negative movements
        let wrapped_position = if new_position < 0 {
            // For negative numbers, we need to handle the wrapping differently
            let abs_new = new_position.abs();
            let remainder = abs_new % len as i32;
            if remainder == 0 {
                0
            } else {
                len as i32 - remainder
            }
        } else {
            new_position % len as i32
        };

        self.cursor_position = wrapped_position as usize;
    }

    /// Move cursor up or down without wrapping (clamps to bounds)
    /// Returns true if the cursor actually moved, false if it was already at the boundary
    pub fn move_cursor_clamped(&mut self, direction: i32) -> bool {
        let len = self.filtered_items.len();
        if len == 0 {
            return false;
        }

        let current_pos = self.cursor_position as i32;
        let new_position = current_pos + direction;

        // Clamp to valid range [0, len-1]
        let clamped_position = new_position.max(0).min(len as i32 - 1) as usize;

        if clamped_position != self.cursor_position {
            self.cursor_position = clamped_position;
            true
        } else {
            false
        }
    }

    /// Toggle selection in multi-select mode
    pub fn toggle_selection(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        let selected_index = self.filtered_indices[self.cursor_position];
        if self.selected_items.contains(&selected_index) {
            self.selected_items.remove(&selected_index);
        } else {
            self.selected_items.insert(selected_index);
        }
    }

    /// Get selected items
    pub fn get_selected_items(&self) -> Vec<(usize, String)> {
        let all_items = self.stream.get_all_items();
        let mut selected: Vec<(usize, String)> = self
            .selected_items
            .iter()
            .map(|&idx| (idx, all_items[idx].clone()))
            .collect();
        // Sort by index to preserve original order
        selected.sort_by_key(|k| k.0);
        selected
    }

    /// Check if an item is selected by its original index
    pub fn is_selected(&self, original_index: usize) -> bool {
        self.selected_items.contains(&original_index)
    }

    /// Set query and update filter
    pub async fn set_query(&mut self, query: String) {
        self.query = query;
        self.update_filter().await;
    }

    /// Get filtered items
    pub fn get_filtered_items(&self) -> &[String] {
        &self.filtered_items
    }

    /// Get the original index for a filtered item at the given position
    pub fn get_original_index(&self, position: usize) -> Option<usize> {
        self.filtered_indices.get(position).cloned()
    }

    /// Get cursor position
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Get current query
    pub fn get_query(&self) -> &str {
        &self.query
    }

    /// Check if multi-select mode is enabled
    pub fn is_multi_select(&self) -> bool {
        self.multi_select
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_fuzzy_finder_new() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let finder = FuzzyFinder::with_items_async(items, false).await;
        assert_eq!(finder.get_query(), "");
        assert_eq!(finder.get_cursor_position(), 0);
        assert!(!finder.multi_select);
    }

    #[tokio::test]
    async fn test_async_fuzzy_finder_update_filter() {
        let items = vec!["apple".to_string(), "banana".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;
        finder.set_query("app".to_string()).await;
        let filtered = finder.get_filtered_items();
        assert!(!filtered.is_empty());
    }

    #[tokio::test]
    async fn test_async_fuzzy_finder_move_cursor() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;
        finder.move_cursor(1);
        assert_eq!(finder.get_cursor_position(), 1);
        finder.move_cursor(1);
        assert_eq!(finder.get_cursor_position(), 2);
        finder.move_cursor(1);
        assert_eq!(finder.get_cursor_position(), 0); // Should wrap
    }

    #[tokio::test]
    async fn test_async_fuzzy_finder_add_items() {
        let items = vec!["apple".to_string()];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;
        let new_items = vec!["banana".to_string(), "cherry".to_string()];
        finder.add_items(new_items).await;
        let all_items = finder.get_filtered_items();
        assert!(all_items.len() >= 3);
    }

    #[tokio::test]
    async fn test_move_cursor_clamped_does_not_wrap() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];
        let mut finder = FuzzyFinder::with_items_async(items, false).await;

        // Start at position 0
        assert_eq!(finder.get_cursor_position(), 0);

        // Move down should work
        assert!(finder.move_cursor_clamped(1));
        assert_eq!(finder.get_cursor_position(), 1);

        // Move to end
        assert!(finder.move_cursor_clamped(1));
        assert_eq!(finder.get_cursor_position(), 2);

        // Try to move past end - should not wrap, should return false
        assert!(!finder.move_cursor_clamped(1));
        assert_eq!(finder.get_cursor_position(), 2); // Still at 2

        // Move back up
        assert!(finder.move_cursor_clamped(-1));
        assert_eq!(finder.get_cursor_position(), 1);

        // Move to beginning
        assert!(finder.move_cursor_clamped(-1));
        assert_eq!(finder.get_cursor_position(), 0);

        // Try to move past beginning - should not wrap, should return false
        assert!(!finder.move_cursor_clamped(-1));
        assert_eq!(finder.get_cursor_position(), 0); // Still at 0
    }
}
