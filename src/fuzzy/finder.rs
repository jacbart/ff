use std::collections::HashMap;
use crate::fuzzy::matching::{sequential_filter, parallel_filter};

#[derive(Debug)]
pub struct FuzzyFinder {
    pub items: Vec<String>,
    pub filtered_items: Vec<String>,
    pub selected_indices: Vec<usize>,
    pub query: String,
    pub cursor_position: usize,
    pub multi_select: bool,
    pub lowercase_items: Vec<String>,
    pub query_cache: HashMap<String, Vec<String>>,
}

impl FuzzyFinder {
    pub fn new(items: Vec<String>, multi_select: bool) -> Self {
        let lowercase_items: Vec<String> = items.iter()
            .map(|item| item.to_lowercase())
            .collect();
        Self {
            filtered_items: items.clone(),
            items,
            selected_indices: Vec::new(),
            query: String::new(),
            cursor_position: 0,
            multi_select,
            lowercase_items,
            query_cache: HashMap::new(),
        }
    }

    pub fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered_items = self.items.clone();
        } else if let Some(cached_results) = self.query_cache.get(&self.query) {
            self.filtered_items = cached_results.clone();
                    } else {
                let query_lower = self.query.to_lowercase();
                if self.items.len() > 1000 {
                    self.filtered_items = parallel_filter(self, &query_lower);
                } else {
                    self.filtered_items = sequential_filter(self, &query_lower);
                }
                self.query_cache.insert(self.query.clone(), self.filtered_items.clone());
            }
        if self.cursor_position >= self.filtered_items.len() {
            self.cursor_position = if self.filtered_items.is_empty() {
                0
            } else {
                self.filtered_items.len() - 1
            };
        }
    }

    pub fn move_cursor(&mut self, direction: i32) {
        let len = self.filtered_items.len();
        if len == 0 {
            return;
        }
        let new_position = self.cursor_position as i32 + direction;
        if new_position < 0 {
            self.cursor_position = len - 1;
        } else if new_position >= len as i32 {
            self.cursor_position = 0;
        } else {
            self.cursor_position = new_position as usize;
        }
    }

    pub fn toggle_selection(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let selected_item = &self.filtered_items[self.cursor_position];
        if let Some(original_index) = self.items.iter().position(|item| item == selected_item) {
            if let Some(pos) = self.selected_indices.iter().position(|&i| i == original_index) {
                self.selected_indices.remove(pos);
            } else {
                self.selected_indices.push(original_index);
            }
        }
    }

    pub fn get_selected_items(&self) -> Vec<String> {
        if self.multi_select {
            self.selected_indices.iter().filter_map(|&i| self.items.get(i).cloned()).collect()
        } else if self.filtered_items.is_empty() {
            Vec::new()
        } else {
            let selected_item = &self.filtered_items[self.cursor_position];
            vec![selected_item.clone()]
        }
    }
} 