use crate::fuzzy::lsh::LSHIndex;
use crate::fuzzy::quicksort::quicksort;
use crate::fuzzy::stream::ItemStream;
use crate::fuzzy::tree::BinaryTree;

/// Match positions for highlighting
#[derive(Debug, Clone)]
pub struct MatchPositions {
    pub positions: Vec<usize>,
    pub score: f64,
}

/// Async fuzzy finder with streaming capabilities
pub struct FuzzyFinder {
    pub(crate) stream: ItemStream,
    pub(crate) lsh_index: LSHIndex,
    pub(crate) tree: BinaryTree,
    pub(crate) query: String,
    pub(crate) filtered_items: Vec<String>,
    pub(crate) match_positions: Vec<MatchPositions>,
    pub(crate) selected_items: std::collections::HashSet<String>,
    pub(crate) cursor_position: usize,
    pub(crate) multi_select: bool,
    pub(crate) query_cache: std::collections::HashMap<String, Vec<String>>,
}

impl FuzzyFinder {
    /// Create a new async fuzzy finder (empty)
    pub fn new(multi_select: bool) -> Self {
        let stream = ItemStream::new();
        let lsh_index = LSHIndex::new(5);
        let tree = BinaryTree::new();
        Self {
            stream,
            lsh_index,
            tree,
            query: String::new(),
            filtered_items: Vec::new(),
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
        for item in &items {
            finder.lsh_index.insert(item.clone());
            finder.tree.insert(item.clone());
        }
        finder.stream.add_items(items).await;
        finder.filtered_items = finder.stream.get_all_items();
        finder
    }

    /// Update the filtered items based on the current query
    pub async fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered_items = self.stream.get_all_items();
            self.match_positions.clear();
        } else if let Some(cached_results) = self.query_cache.get(&self.query) {
            self.filtered_items = cached_results.clone();
            // Recalculate match positions for cached results
            self.calculate_match_positions();
        } else {
            let query_lower = self.query.to_lowercase();

            // Use tree for efficient search
            let mut results = self.tree.search(&query_lower);

            // Sort results using quicksort with enhanced scoring
            quicksort(&mut results, &|a, b| {
                let a_score = self.calculate_enhanced_score(a, &query_lower);
                let b_score = self.calculate_enhanced_score(b, &query_lower);
                b_score
                    .partial_cmp(&a_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            self.filtered_items = results;
            self.calculate_match_positions();

            self.query_cache
                .insert(self.query.clone(), self.filtered_items.clone());
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

    /// Calculate match positions for highlighting
    fn calculate_match_positions(&mut self) {
        self.match_positions.clear();
        let query_lower = self.query.to_lowercase();

        for item in &self.filtered_items {
            let item_lower = item.to_lowercase();
            let positions = self.find_match_positions(&item_lower, &query_lower);
            let score = self.calculate_enhanced_score(item, &query_lower);
            self.match_positions
                .push(MatchPositions { positions, score });
        }
    }

    /// Find positions of matching characters for highlighting
    fn find_match_positions(&self, item: &str, query: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut query_chars = query.chars().peekable();
        let mut item_chars = item.chars().enumerate();

        while let Some(query_char) = query_chars.peek() {
            if let Some((pos, item_char)) = item_chars.next() {
                if item_char == *query_char {
                    positions.push(pos);
                    query_chars.next();
                }
            } else {
                break;
            }
        }

        positions
    }

    /// Get match positions for a specific item index
    pub fn get_match_positions(&self, index: usize) -> Option<&MatchPositions> {
        self.match_positions.get(index)
    }

    /// Add new items asynchronously
    pub async fn add_items(&mut self, new_items: Vec<String>) {
        for item in new_items.iter() {
            self.lsh_index.insert(item.clone());
            self.tree.insert(item.clone());
        }
        self.stream.add_items(new_items).await;
        self.update_filter().await;
    }

    /// Enhanced score calculation for better ranking
    fn calculate_enhanced_score(&self, item: &str, query: &str) -> f64 {
        let item_lower = item.to_lowercase();
        let query_lower = query.to_lowercase();

        if item_lower == query_lower {
            return 1.0;
        }

        if item_lower.starts_with(&query_lower) {
            return 0.9;
        }

        if item_lower.contains(&query_lower) {
            return 0.8;
        }

        // Calculate character sequence score with consecutive bonus
        let mut score = 0.0;
        let mut query_chars = query_lower.chars().peekable();
        let mut item_chars = item_lower.chars();
        let mut consecutive = 0;
        let mut total_matches = 0;

        while let Some(query_char) = query_chars.peek() {
            if let Some(item_char) = item_chars.next() {
                if item_char == *query_char {
                    consecutive += 1;
                    total_matches += 1;
                    query_chars.next();
                } else {
                    consecutive = 0;
                }
            } else {
                break;
            }
        }

        if query_chars.peek().is_none() {
            // Base score for matching all characters
            score = 0.5;

            // Bonus for consecutive matches
            score += consecutive as f64 * 0.1;

            // Bonus for total matches
            score += total_matches as f64 * 0.05;

            // Penalty for length difference
            let length_diff = (item_lower.len() as i32 - query_lower.len() as i32).abs() as f64;
            score -= length_diff * 0.01;
        }

        score
    }

    /// Move cursor up or down
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

    /// Toggle selection in multi-select mode
    pub fn toggle_selection(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }

        let selected_item = self.filtered_items[self.cursor_position].clone();
        if self.selected_items.contains(&selected_item) {
            self.selected_items.remove(&selected_item);
        } else {
            self.selected_items.insert(selected_item);
        }
    }

    /// Get selected items
    pub fn get_selected_items(&self) -> Vec<String> {
        self.selected_items.iter().cloned().collect()
    }

    /// Check if an item is selected
    pub fn is_selected(&self, item: &str) -> bool {
        self.selected_items.contains(item)
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

    /// Get cursor position
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Get current query
    pub fn get_query(&self) -> &str {
        &self.query
    }

    /// Get tree size for performance monitoring
    pub fn get_tree_size(&self) -> usize {
        self.tree.size()
    }

    /// Get tree height for performance monitoring
    pub fn get_tree_height(&self) -> usize {
        self.tree.height()
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
}
