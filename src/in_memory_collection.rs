use std::{collections::HashMap, sync::{Mutex, PoisonError}};
use serde_json::Value;

use crate::id_manager::{IdManager, IdType, IdValue};

#[derive(Debug)]
pub enum CollectionError {
    MutexPoisoned,
    ItemNotFound,
}

impl<T> From<PoisonError<T>> for CollectionError {
    fn from(_: PoisonError<T>) -> Self {
        CollectionError::MutexPoisoned
    }
}

pub type CollectionResult<T> = Result<T, CollectionError>;

pub struct InMemoryCollection {
    db: Mutex<HashMap<String, Value>>,
    id_manager: Mutex<IdManager>,
    id_key: String,
}

impl InMemoryCollection {
    pub fn new(id_type: IdType, id_key: String) -> Self {
        let db: HashMap<String, Value> = HashMap::new();
        let db = Mutex::new(db);

        let id_manager = IdManager::new(id_type);
        let id_manager = Mutex::new(id_manager);
        Self {
            db,
            id_manager,
            id_key
        }
    }

    pub fn get_all(&self) -> Vec<Value> {
        let data = self.db.lock().unwrap();
        data.values().cloned().collect::<Vec<Value>>()
    }

    pub fn get_paginated(&self, offset: usize, limit: usize) -> Vec<Value> {
        let data = self.db.lock().unwrap();
        data.values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect::<Vec<Value>>()
    }

    pub fn get(&self, id: &str) -> Option<Value> {
        let data = self.db.lock().unwrap();
        data.get(id).cloned()
    }

    pub fn exists(&self, id: &str) -> bool {
        let data = self.db.lock().unwrap();
        data.contains_key(id)
    }

    pub fn count(&self) -> usize {
        let data = self.db.lock().unwrap();
        data.len()
    }

    pub fn add(&mut self, item: Value) -> Option<Value> {
        let next_id = {
            self.id_manager.lock().unwrap().next()
        };

        if let Some(id_value) = next_id {
            let mut item = item;

            // Convert IdValue to string and add it to the item
            let id_string = match &id_value {
                IdValue::Int(id) => id.to_string(),
                IdValue::Uuid(uuid) => uuid.clone(),
            };

            // Add the ID to the item using the configured id_key
            if let Value::Object(ref mut map) = item {
                map.insert(self.id_key.clone(), Value::String(id_string.clone()));
            }

            // Lock the database and insert the item
            let mut data = self.db.lock().unwrap();
            data.insert(id_string, item.clone());

            return Some(item);
        }

        None
    }

    pub fn add_batch(&mut self, items: Value) -> Vec<Value> {
        let mut added_items = Vec::new();

        if let Value::Array(items_array) = items {
            // Lock the database once for the entire batch operation
            let mut data = self.db.lock().unwrap();

            for item in items_array {
                if let Value::Object(ref item_map) = item {
                    // Extract the ID from the item using the configured id_key
                    if let Some(Value::String(id)) = item_map.get(&self.id_key) {
                        // Insert the item with its existing ID
                        data.insert(id.clone(), item.clone());
                        added_items.push(item);
                    }
                    // Skip items that don't have the required ID field
                }
                // Skip non-object items
            }
        }

        added_items
    }

    pub fn update(&mut self, id: &str, item: Value) -> Option<Value> {
        let mut item = item;

        // Add the ID to the item using the configured id_key
        if let Value::Object(ref mut map) = item {
            map.insert(self.id_key.clone(), Value::String(id.to_string()));
        }

        // Lock the database and update the item if it exists
        let mut data = self.db.lock().unwrap();
        if data.contains_key(id) {
            data.insert(id.to_string(), item.clone());
            Some(item)
        } else {
            None
        }
    }

    pub fn update_partial(&mut self, id: &str, partial_item: Value) -> Option<Value> {
        // Lock the database and check if item exists
        let mut data = self.db.lock().unwrap();

        if let Some(existing_item) = data.get(id).cloned() {
            // Merge the partial update with the existing item
            let updated_item = Self::merge_json_values(existing_item, partial_item);

            // Ensure the ID is still present in the updated item
            let mut final_item = updated_item;
            if let Value::Object(ref mut map) = final_item {
                map.insert(self.id_key.clone(), Value::String(id.to_string()));
            }

            // Update the item in the database
            data.insert(id.to_string(), final_item.clone());
            Some(final_item)
        } else {
            None
        }
    }

    pub fn delete(&mut self, id: &str) -> Option<Value> {
        // Lock the database and remove the item if it exists
        let mut data = self.db.lock().unwrap();
        data.remove(id)
    }

    pub fn clear(&mut self) -> usize {
        let mut data = self.db.lock().unwrap();
        let count = data.len();
        data.clear();
        count
    }

    fn merge_json_values(mut base: Value, update: Value) -> Value {
        match (&mut base, update) {
            (Value::Object(base_map), Value::Object(update_map)) => {
                // Merge object fields
                for (key, value) in update_map {
                    if base_map.contains_key(&key) {
                        // Recursively merge nested objects
                        let existing_value = base_map.get(&key).unwrap().clone();
                        base_map.insert(key, Self::merge_json_values(existing_value, value));
                    } else {
                        // Add new field
                        base_map.insert(key, value);
                    }
                }
                base
            }
            // For non-object values, replace entirely
            (_, update_value) => update_value,
        }
    }
}