use futures::stream::{self, Stream};
use std::collections::BTreeSet;
use tokio::sync::mpsc;

/// Async stream for processing items
pub struct ItemStream {
    items: BTreeSet<String>,
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
}

impl ItemStream {
    /// Create a new async item stream
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self {
            items: BTreeSet::new(),
            tx,
            rx,
        }
    }

    /// Add items to the stream
    pub async fn add_items(&mut self, new_items: Vec<String>) {
        for item in new_items {
            self.items.insert(item.clone());
            if (self.tx.send(item).await).is_err() {
                break;
            }
        }
    }

    /// Get all items as a stream
    pub fn stream(&self) -> impl Stream<Item = String> + '_ {
        stream::iter(self.items.iter().cloned())
    }

    /// Get filtered items as a stream
    pub fn filtered_stream<F>(&self, filter: F) -> impl Stream<Item = String>
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        let items = self.items.clone();
        stream::iter(items.into_iter().filter(move |item| filter(item)))
    }

    /// Process items asynchronously with a function
    pub async fn process_async<F, Fut, T>(&self, processor: F) -> Vec<T>
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let items = self.items.clone();
        let futures: Vec<_> = items
            .into_iter()
            .map(|item| {
                let processor = &processor;
                async move { processor(item).await }
            })
            .collect();

        futures::future::join_all(futures).await
    }

    /// Get items from the receiver channel
    pub async fn receive(&mut self) -> Option<String> {
        self.rx.recv().await
    }

    /// Get all items as a vector
    pub fn get_all_items(&self) -> Vec<String> {
        self.items.iter().cloned().collect()
    }

    /// Check if stream is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Default for ItemStream {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a stream from a vector of items
pub fn create_stream_from_items(items: Vec<String>) -> impl Stream<Item = String> {
    stream::iter(items)
}

/// Process a vector as an async stream
pub async fn process_vector_as_stream<T, F, Fut, R>(items: Vec<T>, processor: F) -> Vec<R>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let futures: Vec<_> = items
        .into_iter()
        .map(|item| {
            let processor = &processor;
            async move { processor(item).await }
        })
        .collect();

    futures::future::join_all(futures).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use tokio;

    #[tokio::test]
    async fn test_async_item_stream_new() {
        let stream = ItemStream::new();
        assert!(stream.is_empty());
        assert_eq!(stream.len(), 0);
    }

    #[tokio::test]
    async fn test_async_item_stream_add_items() {
        let mut stream = ItemStream::new();
        let items = vec!["apple".to_string(), "banana".to_string()];

        stream.add_items(items).await;
        assert_eq!(stream.len(), 2);
    }

    #[tokio::test]
    async fn test_async_item_stream_filtered() {
        let mut stream = ItemStream::new();
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];

        stream.add_items(items).await;

        let filtered: Vec<String> = stream
            .filtered_stream(|item| item.starts_with('a'))
            .collect()
            .await;

        assert_eq!(filtered, vec!["apple".to_string()]);
    }

    #[tokio::test]
    async fn test_process_vector_as_stream() {
        let items = vec![1, 2, 3, 4, 5];

        let results = process_vector_as_stream(items, |x| async move { x * 2 }).await;
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_stream_receive() {
        let mut stream = ItemStream::new();
        let items = vec!["test".to_string()];

        stream.add_items(items).await;

        if let Some(item) = stream.receive().await {
            assert_eq!(item, "test");
        } else {
            panic!("Expected to receive an item");
        }
    }
}
