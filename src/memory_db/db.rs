use std::collections::HashMap;

use crate::memory_db::{CollectionConfig, LockedMemCollection, MemoryCollection};

#[derive(Default)]
pub struct Db {
    pub collections: HashMap<String, LockedMemCollection>,
}

impl Db {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, config: CollectionConfig) -> &LockedMemCollection {
        use std::collections::hash_map::Entry;
        match self.collections.entry(config.name.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(MemoryCollection::new(config).into_locked()),
        }
    }

    pub fn get(&self, col_name: &str) -> Option<&LockedMemCollection> {
        self.collections.get(col_name)
    }
}
