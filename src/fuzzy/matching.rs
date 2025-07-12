use rayon::prelude::*;
use crate::fuzzy::finder::FuzzyFinder;

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

pub fn sequential_filter(finder: &FuzzyFinder, query_lower: &str) -> Vec<String> {
    let mut results = Vec::new();
    for (i, item) in finder.items.iter().enumerate() {
        if fuzzy_match(&finder.lowercase_items[i], query_lower) {
            results.push(item.clone());
        }
    }
    results
}

pub fn parallel_filter(finder: &FuzzyFinder, query_lower: &str) -> Vec<String> {
    finder.items
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