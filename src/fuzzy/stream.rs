//! Item storage for the fuzzy finder.
//!
//! Items live in a single ordered buffer. An item's position in that buffer is
//! its *original index*, which is what selection, filtering, and output all key
//! on. Items are never duplicated here: earlier versions mirrored every item
//! into an `mpsc` channel that nothing drained, which deadlocked the TUI once
//! 1000 items had accumulated.

/// Ordered buffer of items to search through.
pub struct ItemStream {
    items: Vec<String>,
}

impl ItemStream {
    /// Create an empty item stream.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Append items, preserving insertion order.
    ///
    /// Never blocks: items are appended to the backing buffer only.
    pub fn add_items(&mut self, new_items: Vec<String>) {
        self.items.extend(new_items);
    }

    /// Get all items as an owned vector.
    pub fn get_all_items(&self) -> Vec<String> {
        self.items.clone()
    }

    /// Check if the stream is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Default for ItemStream {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_stream_new() {
        let stream = ItemStream::new();
        assert!(stream.is_empty());
        assert_eq!(stream.len(), 0);
    }

    #[test]
    fn test_item_stream_add_items() {
        let mut stream = ItemStream::new();
        stream.add_items(vec!["apple".to_string(), "banana".to_string()]);

        assert_eq!(stream.len(), 2);
        assert_eq!(
            stream.get_all_items(),
            vec!["apple".to_string(), "banana".to_string()]
        );
    }

    #[test]
    fn test_item_stream_preserves_insertion_order() {
        let mut stream = ItemStream::new();
        stream.add_items(vec!["first".to_string(), "second".to_string()]);
        stream.add_items(vec!["third".to_string()]);

        assert_eq!(
            stream.get_all_items(),
            vec![
                "first".to_string(),
                "second".to_string(),
                "third".to_string()
            ]
        );
    }

    #[test]
    fn test_item_stream_accepts_more_than_1000_items_in_one_batch() {
        let mut stream = ItemStream::new();
        let items: Vec<String> = (0..10_000).map(|i| format!("item{i}")).collect();

        stream.add_items(items);

        assert_eq!(stream.len(), 10_000);
    }

    #[test]
    fn test_item_stream_accepts_repeated_batches() {
        let mut stream = ItemStream::new();
        for batch in 0..10 {
            let items: Vec<String> = (0..1000).map(|i| format!("item{batch}-{i}")).collect();
            stream.add_items(items);
        }

        assert_eq!(stream.len(), 10_000);
        assert!(!stream.is_empty());
    }
}
