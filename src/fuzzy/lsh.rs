use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Type alias for hash functions to reduce complexity
type HashFunction = Box<dyn Fn(&str) -> u64 + Send + Sync>;

/// Locality Sensitive Hashing index for grouping similar items
pub struct LSHIndex {
    buckets: HashMap<u64, Vec<String>>,
    hash_functions: Vec<HashFunction>,
}

impl LSHIndex {
    /// Create a new LSH index with the specified number of hash functions
    pub fn new(num_hashes: usize) -> Self {
        let mut hash_functions: Vec<HashFunction> = Vec::new();

        for i in 0..num_hashes {
            let seed = i as u64;
            let hash_fn: HashFunction = Box::new(move |s: &str| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                seed.hash(&mut hasher);
                s.hash(&mut hasher);
                hasher.finish()
            });
            hash_functions.push(hash_fn);
        }

        Self {
            buckets: HashMap::new(),
            hash_functions,
        }
    }

    /// Add an item to the LSH index
    pub fn insert(&mut self, item: String) {
        for hash_fn in &self.hash_functions {
            let hash = hash_fn(&item);
            self.buckets.entry(hash).or_default().push(item.clone());
        }
    }

    /// Find similar items based on LSH buckets
    pub fn find_similar(&self, query: &str, threshold: usize) -> Vec<String> {
        let mut candidates = HashMap::new();

        for hash_fn in &self.hash_functions {
            let hash = hash_fn(query);
            if let Some(bucket) = self.buckets.get(&hash) {
                for item in bucket {
                    *candidates.entry(item.clone()).or_insert(0) += 1;
                }
            }
        }

        candidates
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .map(|(item, _)| item)
            .collect()
    }

    /// Get all items in the index
    pub fn all_items(&self) -> Vec<String> {
        let mut items = std::collections::HashSet::new();
        for bucket in self.buckets.values() {
            items.extend(bucket.iter().cloned());
        }
        items.into_iter().collect()
    }

    /// Clear all items from the index
    pub fn clear(&mut self) {
        self.buckets.clear();
    }
}

/// Simple similarity function using character overlap
pub fn similarity(a: &str, b: &str) -> f64 {
    let a_chars: std::collections::HashSet<char> = a.chars().collect();
    let b_chars: std::collections::HashSet<char> = b.chars().collect();

    let intersection = a_chars.intersection(&b_chars).count();
    let union = a_chars.union(&b_chars).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsh_index_new() {
        let index = LSHIndex::new(3);
        assert_eq!(index.hash_functions.len(), 3);
        assert!(index.buckets.is_empty());
    }

    #[test]
    fn test_lsh_index_insert() {
        let mut index = LSHIndex::new(2);
        index.insert("test".to_string());

        let all_items = index.all_items();
        assert_eq!(all_items.len(), 1);
        assert!(all_items.contains(&"test".to_string()));
    }

    #[test]
    fn test_lsh_index_find_similar() {
        let mut index = LSHIndex::new(3);
        index.insert("apple".to_string());
        index.insert("application".to_string());
        index.insert("banana".to_string());

        let similar = index.find_similar("apple", 1);
        assert!(!similar.is_empty());
    }

    #[test]
    fn test_similarity_function() {
        assert_eq!(similarity("apple", "apple"), 1.0);
        assert!(similarity("apple", "banana") < 1.0);
        assert!(similarity("apple", "application") > 0.0);
    }

    #[test]
    fn test_lsh_index_clear() {
        let mut index = LSHIndex::new(2);
        index.insert("test".to_string());
        assert!(!index.all_items().is_empty());

        index.clear();
        assert!(index.all_items().is_empty());
    }
}
