use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::memory_db::{CollectionConfig, MemoryCollection, ProtectedMemCollection};

pub type DbProtected = Arc<RwLock<Db>>;

#[derive(Default)]
pub struct Db {
    collections: HashMap<String, ProtectedMemCollection>,
}

impl Db {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_protected(self) -> DbProtected {
        Arc::new(RwLock::new(self))
    }

    pub fn create(&mut self, config: CollectionConfig) -> ProtectedMemCollection {
        let coll_name = config.name.clone();
        let collection = MemoryCollection::new(config).into_protected();
        self.collections.insert(coll_name, Arc::clone(&collection));

        collection
    }

    pub fn get(&self, col_name: &str) -> Option<ProtectedMemCollection> {
        self.collections.get(col_name).map(Arc::clone)
    }
}

pub trait DbProtectedExt {
    fn create(&self, config: CollectionConfig) -> ProtectedMemCollection;
    fn get(&self, col_name: &str) -> Option<ProtectedMemCollection>;
}

impl DbProtectedExt for DbProtected {
    fn create(&self, config: CollectionConfig) -> ProtectedMemCollection {
        self.write().unwrap().create(config)
    }

    fn get(&self, col_name: &str) -> Option<ProtectedMemCollection> {
        self.read().unwrap().get(col_name)
    }
}
