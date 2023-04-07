/// Helper trait to specify which other traits a type must fulfil
pub trait StoreItem: Clone + Send + Sync {}
impl<T> StoreItem for T where T: Clone + Send + Sync {}

/// A simple in memory store of items that implements the DBStore trait
/// Mainly used for tests
pub struct InMemoryStore<K, Item: StoreItem> {
    pub values: std::collections::HashMap<K, Item>,
}

impl<K, Item: StoreItem> InMemoryStore<K, Item> {
    pub fn new() -> Self {
        Self {
            values: std::collections::HashMap::new(),
        }
    }
}
