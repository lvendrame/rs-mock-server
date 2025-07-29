use std::{collections::HashMap};
use serde_json::Value;

use crate::id_manager::{IdManager, IdType, IdValue};

pub struct InMemoryCollection {
    db: HashMap<String, Value>,
    id_manager: IdManager,
    id_key: String,
}

impl InMemoryCollection {
    pub fn new(id_type: IdType, id_key: String) -> Self {
        let db: HashMap<String, Value> = HashMap::new();
        let id_manager = IdManager::new(id_type);
        Self {
            db,
            id_manager,
            id_key
        }
    }

    pub fn get_all(&self) -> Vec<Value> {
        self.db.values().cloned().collect::<Vec<Value>>()
    }

    pub fn get_paginated(&self, offset: usize, limit: usize) -> Vec<Value> {
        self.db.values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect::<Vec<Value>>()
    }

    pub fn get(&self, id: &str) -> Option<Value> {
        self.db.get(id).cloned()
    }

    pub fn exists(&self, id: &str) -> bool {
        self.db.contains_key(id)
    }

    pub fn count(&self) -> usize {
        self.db.len()
    }

    pub fn add(&mut self, item: Value) -> Option<Value> {
        let next_id = {
            self.id_manager.next()
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

            self.db.insert(id_string, item.clone());

            return Some(item);
        }

        None
    }

    pub fn add_batch(&mut self, items: Value) -> Vec<Value> {
        let mut added_items = Vec::new();

        if let Value::Array(items_array) = items {
            let mut max_id = None;
            for item in items_array {
                if let Value::Object(ref item_map) = item {
                    let id = item_map.get(&self.id_key);
                    let id = match self.id_manager.id_type {
                        IdType::Uuid => match id {
                            Some(Value::String(id)) => Some(id.clone()),
                            _ => None
                        },
                        IdType::Int => match id {
                            Some(Value::Number(id)) => {
                                if let Some(current) = max_id {
                                    let id = id.as_u64().unwrap();
                                    if current < id {
                                        max_id = Some(id);
                                    }
                                } else {
                                    max_id = id.as_u64();
                                }
                                Some(id.to_string())
                            },
                            _ => None
                        },
                    };

                    // Extract the ID from the item using the configured id_key
                    if let Some(id) = id {
                        // Insert the item with its existing ID
                        self.db.insert(id.clone(), item.clone());
                        added_items.push(item);
                    }
                    // Skip items that don't have the required ID field
                }
                // Skip non-object items
            }

            // update the id_manager with the max id for an integer id
            if let Some(value) = max_id {
                self.id_manager.set_current(IdValue::Int(value));
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

        if self.db.contains_key(id) {
            self.db.insert(id.to_string(), item.clone());
            Some(item)
        } else {
            None
        }
    }

    pub fn update_partial(&mut self, id: &str, partial_item: Value) -> Option<Value> {
        if let Some(existing_item) = self.db.get(id).cloned() {
            // Merge the partial update with the existing item
            let updated_item = Self::merge_json_values(existing_item, partial_item);

            // Ensure the ID is still present in the updated item
            let mut final_item = updated_item;
            if let Value::Object(ref mut map) = final_item {
                map.insert(self.id_key.clone(), Value::String(id.to_string()));
            }

            // Update the item in the database
            self.db.insert(id.to_string(), final_item.clone());
            Some(final_item)
        } else {
            None
        }
    }

    pub fn delete(&mut self, id: &str) -> Option<Value> {
        self.db.remove(id)
    }

    pub fn clear(&mut self) -> usize {
        let count = self.db.len();
        self.db.clear();
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