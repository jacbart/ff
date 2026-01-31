use std::cmp::Ordering;

/// Binary tree node for efficient item storage
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub item: String,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
    pub count: usize,
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(item: String) -> Self {
        Self {
            item,
            left: None,
            right: None,
            count: 1,
        }
    }

    /// Insert an item into the tree
    pub fn insert(&mut self, item: String) {
        match item.cmp(&self.item) {
            Ordering::Less => {
                if let Some(ref mut left) = self.left {
                    left.insert(item);
                } else {
                    self.left = Some(Box::new(TreeNode::new(item)));
                }
            }
            Ordering::Greater => {
                if let Some(ref mut right) = self.right {
                    right.insert(item);
                } else {
                    self.right = Some(Box::new(TreeNode::new(item)));
                }
            }
            Ordering::Equal => {
                self.count += 1;
            }
        }
    }

    /// Search for items that match a query
    pub fn search(&self, query: &str, results: &mut Vec<String>) {
        let item_lower = self.item.to_lowercase();
        let query_lower = query.to_lowercase();

        // Check if current item matches
        if self.fuzzy_match(&item_lower, &query_lower) {
            results.push(self.item.clone());
        }

        // Recursively search left and right subtrees
        if let Some(ref left) = self.left {
            left.search(query, results);
        }
        if let Some(ref right) = self.right {
            right.search(query, results);
        }
    }

    /// Fuzzy match implementation for tree search
    fn fuzzy_match(&self, item: &str, query: &str) -> bool {
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

    /// Get all items in sorted order
    pub fn get_all_items(&self) -> Vec<String> {
        let mut items = Vec::new();
        self.inorder_traversal(&mut items);
        items
    }

    /// Inorder traversal to get sorted items
    fn inorder_traversal(&self, items: &mut Vec<String>) {
        if let Some(ref left) = self.left {
            left.inorder_traversal(items);
        }
        items.push(self.item.clone());
        if let Some(ref right) = self.right {
            right.inorder_traversal(items);
        }
    }

    /// Get the height of the tree
    pub fn height(&self) -> usize {
        let left_height = self.left.as_ref().map_or(0, |left| left.height());
        let right_height = self.right.as_ref().map_or(0, |right| right.height());
        1 + left_height.max(right_height)
    }

    /// Get the number of nodes in the tree
    pub fn size(&self) -> usize {
        let left_size = self.left.as_ref().map_or(0, |left| left.size());
        let right_size = self.right.as_ref().map_or(0, |right| right.size());
        1 + left_size + right_size
    }
}

/// Binary tree for efficient item storage and retrieval
#[derive(Debug, Clone)]
pub struct BinaryTree {
    root: Option<Box<TreeNode>>,
}

impl BinaryTree {
    /// Create a new empty binary tree
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Insert an item into the tree
    pub fn insert(&mut self, item: String) {
        if let Some(ref mut root) = self.root {
            root.insert(item);
        } else {
            self.root = Some(Box::new(TreeNode::new(item)));
        }
    }

    /// Search for items that match a query
    pub fn search(&self, query: &str) -> Vec<String> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            root.search(query, &mut results);
        }
        results
    }

    /// Get all items in sorted order
    pub fn get_all_items(&self) -> Vec<String> {
        if let Some(ref root) = self.root {
            root.get_all_items()
        } else {
            Vec::new()
        }
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Get the height of the tree
    pub fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.height())
    }

    /// Get the number of nodes in the tree
    pub fn size(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.size())
    }

    /// Clear all items from the tree
    pub fn clear(&mut self) {
        self.root = None;
    }
}

impl Default for BinaryTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_new() {
        let node = TreeNode::new("test".to_string());
        assert_eq!(node.item, "test");
        assert_eq!(node.count, 1);
        assert!(node.left.is_none());
        assert!(node.right.is_none());
    }

    #[test]
    fn test_tree_node_insert() {
        let mut node = TreeNode::new("banana".to_string());
        node.insert("apple".to_string());
        node.insert("cherry".to_string());

        assert!(node.left.is_some());
        assert!(node.right.is_some());
        assert_eq!(node.left.as_ref().unwrap().item, "apple");
        assert_eq!(node.right.as_ref().unwrap().item, "cherry");
    }

    #[test]
    fn test_tree_node_duplicate_insert() {
        let mut node = TreeNode::new("test".to_string());
        node.insert("test".to_string());
        assert_eq!(node.count, 2);
    }

    #[test]
    fn test_tree_node_search() {
        let mut node = TreeNode::new("banana".to_string());
        node.insert("apple".to_string());
        node.insert("cherry".to_string());

        let mut results = Vec::new();
        node.search("app", &mut results);
        assert_eq!(results, vec!["apple"]);
    }

    #[test]
    fn test_tree_node_get_all_items() {
        let mut node = TreeNode::new("banana".to_string());
        node.insert("apple".to_string());
        node.insert("cherry".to_string());

        let items = node.get_all_items();
        assert_eq!(items, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_binary_tree_new() {
        let tree = BinaryTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.size(), 0);
    }

    #[test]
    fn test_binary_tree_insert() {
        let mut tree = BinaryTree::new();
        tree.insert("banana".to_string());
        tree.insert("apple".to_string());
        tree.insert("cherry".to_string());

        assert!(!tree.is_empty());
        assert_eq!(tree.size(), 3);
    }

    #[test]
    fn test_binary_tree_search() {
        let mut tree = BinaryTree::new();
        tree.insert("banana".to_string());
        tree.insert("apple".to_string());
        tree.insert("cherry".to_string());

        let results = tree.search("app");
        assert_eq!(results, vec!["apple"]);
    }

    #[test]
    fn test_binary_tree_get_all_items() {
        let mut tree = BinaryTree::new();
        tree.insert("banana".to_string());
        tree.insert("apple".to_string());
        tree.insert("cherry".to_string());

        let items = tree.get_all_items();
        assert_eq!(items, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_binary_tree_clear() {
        let mut tree = BinaryTree::new();
        tree.insert("test".to_string());
        assert!(!tree.is_empty());

        tree.clear();
        assert!(tree.is_empty());
        assert_eq!(tree.size(), 0);
    }
}
