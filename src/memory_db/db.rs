use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::memory_db::{CollectionConfig, DbCollection, MemoryCollection};

pub type Db = Arc<RwLock<InternalDb>>;

#[derive(Default)]
pub struct InternalDb {
    collections: HashMap<String, MemoryCollection>,
}

impl InternalDb {

    pub fn into_protected(self) -> Db {
        Arc::new(RwLock::new(self))
    }
}

pub trait DbCommon {
    fn new_db() -> Self;
    fn create(&mut self, config: CollectionConfig) -> MemoryCollection;
    fn get(&self, col_name: &str) -> Option<MemoryCollection>;
    fn list_collections(&self) -> Vec<String>;
}

impl DbCommon for InternalDb {

    fn new_db() -> Self {
        Self::default()
    }

    fn create(&mut self, config: CollectionConfig) -> MemoryCollection {
        let coll_name = config.name.clone();
        let collection = MemoryCollection::new_coll(config);
        self.collections.insert(coll_name, Arc::clone(&collection));

        collection
    }

    fn get(&self, col_name: &str) -> Option<MemoryCollection> {
        self.collections.get(col_name).map(Arc::clone)
    }

    fn list_collections(&self) -> Vec<String> {
        self.collections.keys().cloned().collect::<Vec<_>>()
    }

}

impl DbCommon for Db {

    fn new_db() -> Self {
        InternalDb::new_db().into_protected()
    }

    fn create(&mut self, config: CollectionConfig) -> MemoryCollection {
        self.write().unwrap().create(config)
    }

    fn get(&self, col_name: &str) -> Option<MemoryCollection> {
        self.read().unwrap().get(col_name)
    }

    fn list_collections(&self) -> Vec<String> {
        self.read().unwrap().list_collections()
    }
}
