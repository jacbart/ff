use crate::fuzzy::finder::FuzzyFinder;
use rayon::prelude::*;

/// Check if an item matches a query using fuzzy matching.
pub fn fuzzy_match(item: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    if item.contains(query) {
        return true;
    }
    let mut query_chars = query.chars().peekable();
    let mut item_chars = item.chars();
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
}

/// Find the positions of matched characters for highlighting.
pub fn find_match_positions(item: &str, query: &str) -> Vec<usize> {
    if query.is_empty() {
        return vec![];
    }

    let query_lower = query.to_lowercase();
    let item_lower = item.to_lowercase();

    let mut positions = Vec::new();
    let mut query_chars = query_lower.chars().peekable();
    let mut item_chars = item_lower.chars().enumerate();

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

/// Filter items sequentially using fuzzy matching.
pub fn sequential_filter(finder: &FuzzyFinder, query_lower: &str) -> Vec<String> {
    let mut results = Vec::new();
    for (i, item) in finder.items.iter().enumerate() {
        if fuzzy_match(&finder.lowercase_items[i], query_lower) {
            results.push(item.clone());
        }
    }
    results
}

/// Filter items in parallel using fuzzy matching.
pub fn parallel_filter(finder: &FuzzyFinder, query_lower: &str) -> Vec<String> {
    finder
        .items
        .par_iter()
        .enumerate()
        .filter_map(|(i, item)| {
            if fuzzy_match(&finder.lowercase_items[i], query_lower) {
                Some(item.clone())
            } else {
                None
            }
        })
        .collect()
}
