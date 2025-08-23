use std::{collections::HashMap, ffi::OsString, fs, sync::{Arc, RwLock}};
use serde_json::Value;

use crate::memory_db::{constraint::Constraint, id_manager::{IdManager, IdType, IdValue}, CollectionConfig, Criteria, CriteriaBuilder};

pub type MemoryCollection = Arc<RwLock<InternalMemoryCollection>>;

pub struct InternalMemoryCollection {
    collection: HashMap<String, Value>,
    id_manager: IdManager,
    pub config: CollectionConfig,
}

impl InternalMemoryCollection {
    pub fn new(config: CollectionConfig) -> Self {
        let db: HashMap<String, Value> = HashMap::new();
        let id_manager = IdManager::new(config.id_type);
        Self {
            collection: db,
            id_manager,
            config,
        }
    }

    pub fn into_protected(self) -> MemoryCollection {
        Arc::new(RwLock::new(self))
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

pub trait DbCollection {
    fn new_coll(config: CollectionConfig) -> Self;

    fn get_all(&self) -> Vec<Value>;

    fn get_paginated(&self, offset: usize, limit: usize) -> Vec<Value>;

    fn get(&self, id: &str) -> Option<Value>;

    fn get_from_criteria(&self, criteria: &Criteria) -> Vec<Value>;

    fn get_from_where(&self, where_statement: &str) -> Vec<Value>;

    fn get_from_constraint(&self, criteria: &Constraint) -> Vec<Value>;

    fn exists(&self, id: &str) -> bool;

    fn count(&self) -> usize;

    fn add(&mut self, item: Value) -> Option<Value>;

    fn add_batch(&mut self, items: Value) -> Vec<Value>;

    fn update(&mut self, id: &str, item: Value) -> Option<Value>;

    fn update_partial(&mut self, id: &str, partial_item: Value) -> Option<Value>;

    fn delete(&mut self, id: &str) -> Option<Value>;

    fn clear(&mut self) -> usize;

    fn load_from_json(&mut self, json_value: Value) -> Result<Vec<Value>, String>;

    fn load_from_file(&mut self, file_path: &OsString) -> Result<String, String>;
}

impl DbCollection for InternalMemoryCollection {
    fn new_coll(config: CollectionConfig) -> Self {
        Self::new(config)
    }

    fn get_all(&self) -> Vec<Value> {
        self.collection.values().cloned().collect::<Vec<Value>>()
    }

    fn get_paginated(&self, offset: usize, limit: usize) -> Vec<Value> {
        self.collection.values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect::<Vec<Value>>()
    }

    fn get(&self, id: &str) -> Option<Value> {
        self.collection.get(id).cloned()
    }

    fn get_from_constraint(&self, constraint: &Constraint) -> Vec<Value> {
        self.collection.values().filter(|&item| {
                match item {
                    Value::Object(map) => constraint.compare_item(map),
                    _ => false
                }
            }).cloned()
            .collect::<Vec<Value>>()
    }

    fn get_from_criteria(&self, criteria: &Criteria) -> Vec<Value> {
        self.collection.values().filter(|&item| {
                match item {
                    Value::Object(map) => criteria.compare_item(map),
                    _ => false
                }
            }).cloned()
            .collect::<Vec<Value>>()
    }

    fn get_from_where(&self, where_statement: &str) -> Vec<Value> {
        let criteria = CriteriaBuilder::start(where_statement);
        if criteria.is_err() {
            return vec![];
        }

        self.get_from_criteria(criteria.unwrap().as_ref())
    }

    fn exists(&self, id: &str) -> bool {
        self.collection.contains_key(id)
    }

    fn count(&self) -> usize {
        self.collection.len()
    }

    fn add(&mut self, item: Value) -> Option<Value> {
        let next_id = {
            self.id_manager.next()
        };

        let mut item = item;
        let id_string = if let Some(id_value) = next_id {
            // Convert IdValue to string and add it to the item
            let id_string = id_value.to_string();

            // Add the ID to the item using the configured id_key
            if let Value::Object(ref mut map) = item {
                map.insert(self.config.id_key.clone(), Value::String(id_string.clone()));
            }
            Some(id_string)
        } else if let Some(Value::String(id_string)) = item.get(self.config.id_key.clone()){
            Some(id_string.clone())
        } else if let Some(Value::Number(id_number)) = item.get(self.config.id_key.clone()){
            Some(id_number.to_string())
        }else {
            None
        };

        if let Some(id_string) = id_string {
            self.collection.insert(id_string, item.clone());

            return Some(item);
        }

        None
    }

    fn add_batch(&mut self, items: Value) -> Vec<Value> {
        let mut added_items = Vec::new();

        if let Value::Array(items_array) = items {
            let mut max_id = None;
            for item in items_array {
                if let Value::Object(ref item_map) = item {
                    let id = item_map.get(&self.config.id_key);
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
                        IdType::None => match item.get(self.config.id_key.clone()) {
                            Some(Value::String(id_string)) => Some(id_string.clone()),
                            Some(Value::Number(id_number)) => Some(id_number.to_string()),
                            _ => None
                        }
                    };

                    // Extract the ID from the item using the configured id_key
                    if let Some(id) = id {
                        // Insert the item with its existing ID
                        self.collection.insert(id.clone(), item.clone());
                        added_items.push(item);
                    }
                    // Skip items that don't have the required ID field
                }
                // Skip non-object items
            }

            // update the id_manager with the max id for an integer id
            if let Some(value) = max_id {
                if self.id_manager.set_current(IdValue::Int(value)).is_err() {
                    println!("Error to set the value {} to {} collection Id", value, self.config.name.clone());
                }
            }
        }

        added_items
    }

    fn update(&mut self, id: &str, item: Value) -> Option<Value> {
        let mut item = item;

        // Add the ID to the item using the configured id_key
        if let Value::Object(ref mut map) = item {
            map.insert(self.config.id_key.clone(), Value::String(id.to_string()));
        }

        if self.collection.contains_key(id) {
            self.collection.insert(id.to_string(), item.clone());
            Some(item)
        } else {
            None
        }
    }

    fn update_partial(&mut self, id: &str, partial_item: Value) -> Option<Value> {
        if let Some(existing_item) = self.collection.get(id).cloned() {
            // Merge the partial update with the existing item
            let updated_item = Self::merge_json_values(existing_item, partial_item);

            // Ensure the ID is still present in the updated item
            let mut final_item = updated_item;
            if let Value::Object(ref mut map) = final_item {
                map.insert(self.config.id_key.clone(), Value::String(id.to_string()));
            }

            // Update the item in the database
            self.collection.insert(id.to_string(), final_item.clone());
            Some(final_item)
        } else {
            None
        }
    }

    fn delete(&mut self, id: &str) -> Option<Value> {
        self.collection.remove(id)
    }

    fn clear(&mut self) -> usize {
        let count = self.collection.len();
        self.collection.clear();
        count
    }

    fn load_from_json(&mut self, json_value: Value) -> Result<Vec<Value>, String> {
        // Guard: Check if it's a JSON Array
        let Value::Array(_) = json_value else {
            return Err("⚠️ Informed JSON does not contain a JSON array in the root, skipping initial data load".to_string());
        };

        // Load the array into the collection using add_batch
        let added_items = self.add_batch(json_value);
        Ok(added_items)
    }

    fn load_from_file(&mut self, file_path: &OsString) -> Result<String, String> {
        let file_path_lossy = file_path.to_string_lossy();

        // Guard: Try to read the file content
        let file_content = fs::read_to_string(file_path)
            .map_err(|_| format!("⚠️ Could not read file {}, skipping initial data load", file_path_lossy))?;

        // Guard: Try to parse the content as JSON
        let json_value = serde_json::from_str::<Value>(&file_content)
            .map_err(|_| format!("⚠️ File {} does not contain valid JSON, skipping initial data load", file_path_lossy))?;

        match self.load_from_json(json_value) {
            Ok(added_items) => Ok(format!("✔️ Loaded {} initial items from {}", added_items.len(), file_path_lossy)),
            Err(error) => Err(format!("Error to process the file {}. Details: {}", file_path_lossy, error)),
        }
    }
}

impl DbCollection for MemoryCollection {
    fn new_coll(config: CollectionConfig) -> Self {
        InternalMemoryCollection::new_coll(config).into_protected()
    }

    fn get_all(&self) -> Vec<Value> {
        self.read().unwrap().get_all()
    }

    fn get_paginated(&self, offset: usize, limit: usize) -> Vec<Value> {
        self.read().unwrap().get_paginated(offset, limit)
    }

    fn get(&self, id: &str) -> Option<Value> {
        self.read().unwrap().get(id)
    }

    fn get_from_constraint(&self, constraint: &Constraint) -> Vec<Value> {
        self.read().unwrap().get_from_constraint(constraint)
    }

    fn get_from_criteria(&self, criteria: &Criteria) -> Vec<Value> {
        self.read().unwrap().get_from_criteria(criteria)
    }

    fn get_from_where(&self, where_statement: &str) -> Vec<Value> {
        self.read().unwrap().get_from_where(where_statement)
    }

    fn exists(&self, id: &str) -> bool {
        self.read().unwrap().exists(id)
    }

    fn count(&self) -> usize {
        self.read().unwrap().count()
    }

    fn add(&mut self, item: Value) -> Option<Value> {
        self.write().unwrap().add(item)
    }

    fn add_batch(&mut self, items: Value) -> Vec<Value> {
        self.write().unwrap().add_batch(items)
    }

    fn update(&mut self, id: &str, item: Value) -> Option<Value> {
        self.write().unwrap().update(id, item)
    }

    fn update_partial(&mut self, id: &str, partial_item: Value) -> Option<Value> {
        self.write().unwrap().update_partial(id, partial_item)
    }

    fn delete(&mut self, id: &str) -> Option<Value> {
        self.write().unwrap().delete(id)
    }

    fn clear(&mut self) -> usize {
        self.write().unwrap().clear()
    }

    fn load_from_json(&mut self, json_value: Value) -> Result<Vec<Value>, String> {
        self.write().unwrap().load_from_json(json_value)
    }

    fn load_from_file(&mut self, file_path: &OsString) -> Result<String, String> {
        self.write().unwrap().load_from_file(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::memory_db::constraint::Comparer;

    fn create_test_collection() -> InternalMemoryCollection {
        InternalMemoryCollection::new(CollectionConfig::int("id", "test_collection"))
    }

    fn create_uuid_collection() -> InternalMemoryCollection {
        InternalMemoryCollection::new(CollectionConfig::uuid("id", "uuid_collection"))
    }

    fn create_none_collection() -> InternalMemoryCollection {
        InternalMemoryCollection::new(CollectionConfig::none("id", "none_collection"))
    }

    #[test]
    fn test_new_collection() {
        let collection = create_test_collection();
        assert_eq!(collection.count(), 0);
        assert_eq!(collection.config.id_key, "id");
        assert_eq!(collection.config.name, "test_collection");
    }

    #[test]
    fn test_into_protected() {
        let collection = create_test_collection();
        let protected = collection.into_protected();

        let guard = protected.read().unwrap();
        assert_eq!(guard.count(), 0);
        assert_eq!(guard.config.name, "test_collection");
    }

    #[test]
    fn test_get_all_empty() {
        let collection = create_test_collection();
        let all_items = collection.get_all();
        assert!(all_items.is_empty());
    }

    #[test]
    fn test_get_all_with_items() {
        let mut collection = create_test_collection();

        // Add some items
        collection.add(json!({"name": "Item 1"}));
        collection.add(json!({"name": "Item 2"}));
        collection.add(json!({"name": "Item 3"}));

        let all_items = collection.get_all();
        assert_eq!(all_items.len(), 3);

        // Check that all items have IDs assigned
        for item in &all_items {
            assert!(item.get("id").is_some());
            assert!(item.get("name").is_some());
        }
    }

    #[test]
    fn test_get_existing_item() {
        let mut collection = create_test_collection();
        let item = collection.add(json!({"name": "Test Item"})).unwrap();
        let id = item.get("id").unwrap().as_str().unwrap();

        let retrieved = collection.get(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().get("name").unwrap(), "Test Item");
    }

    #[test]
    fn test_get_nonexistent_item() {
        let collection = create_test_collection();
        let retrieved = collection.get("999");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_get_paginated_empty() {
        let collection = create_test_collection();
        let paginated = collection.get_paginated(0, 10);
        assert!(paginated.is_empty());
    }

    #[test]
    fn test_get_paginated_with_items() {
        let mut collection = create_test_collection();

        // Add 10 items
        for i in 1..=10 {
            collection.add(json!({"name": format!("Item {}", i)}));
        }

        // Test first page
        let first_page = collection.get_paginated(0, 3);
        assert_eq!(first_page.len(), 3);

        // Test second page
        let second_page = collection.get_paginated(3, 3);
        assert_eq!(second_page.len(), 3);

        // Test last page (partial)
        let last_page = collection.get_paginated(9, 5);
        assert_eq!(last_page.len(), 1);

        // Test beyond range
        let empty_page = collection.get_paginated(15, 5);
        assert!(empty_page.is_empty());
    }

    #[test]
    fn test_exists() {
        let mut collection = create_test_collection();
        let item = collection.add(json!({"name": "Test Item"})).unwrap();
        let id = item.get("id").unwrap().as_str().unwrap();

        assert!(collection.exists(id));
        assert!(!collection.exists("999"));
    }

    #[test]
    fn test_count() {
        let mut collection = create_test_collection();
        assert_eq!(collection.count(), 0);

        collection.add(json!({"name": "Item 1"}));
        assert_eq!(collection.count(), 1);

        collection.add(json!({"name": "Item 2"}));
        assert_eq!(collection.count(), 2);

        // Delete one
        let all_items = collection.get_all();
        let id = all_items[0].get("id").unwrap().as_str().unwrap();
        collection.delete(id);
        assert_eq!(collection.count(), 1);
    }

    #[test]
    fn test_add_with_int_id() {
        let mut collection = create_test_collection();

        let item = collection.add(json!({"name": "Test Item"}));
        assert!(item.is_some());

        let item = item.unwrap();
        assert_eq!(item.get("name").unwrap(), "Test Item");
        assert_eq!(item.get("id").unwrap(), "1");

        // Add another item
        let item2 = collection.add(json!({"name": "Test Item 2"})).unwrap();
        assert_eq!(item2.get("id").unwrap(), "2");
    }

    #[test]
    fn test_add_with_uuid_id() {
        let mut collection = create_uuid_collection();

        let item = collection.add(json!({"name": "Test Item"}));
        assert!(item.is_some());

        let item = item.unwrap();
        assert_eq!(item.get("name").unwrap(), "Test Item");
        let id = item.get("id").unwrap().as_str().unwrap();
        assert!(!id.is_empty());
        assert!(id.len() > 10); // UUIDs are longer than 10 characters
    }

    #[test]
    fn test_add_with_none_id_existing() {
        let mut collection = create_none_collection();

        let item = collection.add(json!({"id": "custom-id", "name": "Test Item"}));
        assert!(item.is_some());

        let item = item.unwrap();
        assert_eq!(item.get("name").unwrap(), "Test Item");
        assert_eq!(item.get("id").unwrap(), "custom-id");
    }

    #[test]
    fn test_add_with_none_id_number_existing() {
        let mut collection = create_none_collection();

        let item = collection.add(json!({"id": 1, "name": "Test Item"}));
        assert!(item.is_some());

        let item = item.unwrap();
        assert_eq!(item.get("name").unwrap(), "Test Item");
        assert_eq!(item.get("id").unwrap(), 1);
    }

    #[test]
    fn test_add_with_none_id_missing() {
        let mut collection = create_none_collection();

        let item = collection.add(json!({"name": "Test Item"}));
        assert!(item.is_none());
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_add_batch_int() {
        let mut collection = create_test_collection();

        let batch = json!([
            {"name": "Item 1"},
            {"id": 5, "name": "Item 2"},
            {"id": 3, "name": "Item 3"},
            {"id": 10, "name": "Item 4"}
        ]);

        let added_items = collection.add_batch(batch);
        assert_eq!(added_items.len(), 3); // Only items with IDs should be added
        assert_eq!(collection.count(), 3);

        // Check that the max ID was set correctly
        let new_item = collection.add(json!({"name": "New Item"})).unwrap();
        assert_eq!(new_item.get("id").unwrap(), "11"); // Should be max + 1
    }

    #[test]
    fn test_add_batch_uuid() {
        let mut collection = create_uuid_collection();

        let batch = json!([
            {"id": "uuid-1", "name": "Item 1"},
            {"id": "uuid-2", "name": "Item 2"},
            {"name": "Item 3"} // This should be skipped
        ]);

        let added_items = collection.add_batch(batch);
        assert_eq!(added_items.len(), 2);
        assert_eq!(collection.count(), 2);
    }

    #[test]
    fn test_add_batch_none() {
        let mut collection = create_none_collection();

        let batch = json!([
            {"id": "custom-1", "name": "Item 1"},
            {"id": "custom-2", "name": "Item 2"},
            {"name": "Item 3"}, // This should be skipped
            {"id": 3, "name": "Item 4"},
        ]);

        let added_items = collection.add_batch(batch);
        assert_eq!(added_items.len(), 3);
        assert_eq!(collection.count(), 3);
    }

    #[test]
    fn test_add_batch_non_array() {
        let mut collection = create_test_collection();

        let non_array = json!({"name": "Single Item"});
        let added_items = collection.add_batch(non_array);
        assert!(added_items.is_empty());
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_update_existing_item() {
        let mut collection = create_test_collection();
        let item = collection.add(json!({"name": "Original Name"})).unwrap();
        let id = item.get("id").unwrap().as_str().unwrap();

        let updated = collection.update(id, json!({"name": "Updated Name", "description": "New field"}));
        assert!(updated.is_some());

        let updated_item = updated.unwrap();
        assert_eq!(updated_item.get("name").unwrap(), "Updated Name");
        assert_eq!(updated_item.get("description").unwrap(), "New field");
        assert_eq!(updated_item.get("id").unwrap(), id);

        // Verify it's actually updated in the collection
        let retrieved = collection.get(id).unwrap();
        assert_eq!(retrieved.get("name").unwrap(), "Updated Name");
    }

    #[test]
    fn test_update_nonexistent_item() {
        let mut collection = create_test_collection();

        let updated = collection.update("999", json!({"name": "Updated Name"}));
        assert!(updated.is_none());
    }

    #[test]
    fn test_update_partial_existing_item() {
        let mut collection = create_test_collection();
        let item = collection.add(json!({
            "name": "Original Name",
            "description": "Original Description",
            "count": 42
        })).unwrap();
        let id = item.get("id").unwrap().as_str().unwrap();

        let updated = collection.update_partial(id, json!({"name": "Updated Name"}));
        assert!(updated.is_some());

        let updated_item = updated.unwrap();
        assert_eq!(updated_item.get("name").unwrap(), "Updated Name");
        assert_eq!(updated_item.get("description").unwrap(), "Original Description"); // Should remain
        assert_eq!(updated_item.get("count").unwrap(), 42); // Should remain
        assert_eq!(updated_item.get("id").unwrap(), id);
    }

    #[test]
    fn test_update_partial_nested_objects() {
        let mut collection = create_test_collection();
        let item = collection.add(json!({
            "name": "Test Item",
            "config": {
                "enabled": true,
                "timeout": 30,
                "nested": {
                    "value": "original"
                }
            }
        })).unwrap();
        let id = item.get("id").unwrap().as_str().unwrap();

        let updated = collection.update_partial(id, json!({
            "config": {
                "timeout": 60,
                "nested": {
                    "value": "updated",
                    "new_field": "added"
                }
            }
        }));

        assert!(updated.is_some());
        let updated_item = updated.unwrap();

        let config = updated_item.get("config").unwrap();
        assert_eq!(config.get("enabled").unwrap(), true); // Should remain
        assert_eq!(config.get("timeout").unwrap(), 60); // Should be updated

        let nested = config.get("nested").unwrap();
        assert_eq!(nested.get("value").unwrap(), "updated");
        assert_eq!(nested.get("new_field").unwrap(), "added");
    }

    #[test]
    fn test_update_partial_nonexistent_item() {
        let mut collection = create_test_collection();

        let updated = collection.update_partial("999", json!({"name": "Updated Name"}));
        assert!(updated.is_none());
    }

    #[test]
    fn test_delete_existing_item() {
        let mut collection = create_test_collection();
        let item = collection.add(json!({"name": "Test Item"})).unwrap();
        let id = item.get("id").unwrap().as_str().unwrap();

        assert_eq!(collection.count(), 1);

        let deleted = collection.delete(id);
        assert!(deleted.is_some());
        assert_eq!(deleted.unwrap().get("name").unwrap(), "Test Item");
        assert_eq!(collection.count(), 0);
        assert!(!collection.exists(id));
    }

    #[test]
    fn test_delete_nonexistent_item() {
        let mut collection = create_test_collection();

        let deleted = collection.delete("999");
        assert!(deleted.is_none());
    }

    #[test]
    fn test_clear_empty_collection() {
        let mut collection = create_test_collection();

        let count = collection.clear();
        assert_eq!(count, 0);
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_clear_with_items() {
        let mut collection = create_test_collection();

        // Add some items
        collection.add(json!({"name": "Item 1"}));
        collection.add(json!({"name": "Item 2"}));
        collection.add(json!({"name": "Item 3"}));

        assert_eq!(collection.count(), 3);

        let count = collection.clear();
        assert_eq!(count, 3);
        assert_eq!(collection.count(), 0);
        assert!(collection.get_all().is_empty());
    }

    #[test]
    fn test_merge_json_values_objects() {
        let base = json!({
            "name": "Original",
            "config": {
                "enabled": true,
                "timeout": 30
            },
            "tags": ["tag1", "tag2"]
        });

        let update = json!({
            "name": "Updated",
            "config": {
                "timeout": 60,
                "new_setting": "value"
            },
            "description": "New field"
        });

        let merged = InternalMemoryCollection::merge_json_values(base, update);

        assert_eq!(merged.get("name").unwrap(), "Updated");
        assert_eq!(merged.get("description").unwrap(), "New field");
        assert_eq!(merged.get("tags").unwrap(), &json!(["tag1", "tag2"])); // Should remain

        let config = merged.get("config").unwrap();
        assert_eq!(config.get("enabled").unwrap(), true); // Should remain
        assert_eq!(config.get("timeout").unwrap(), 60); // Should be updated
        assert_eq!(config.get("new_setting").unwrap(), "value"); // Should be added
    }

    #[test]
    fn test_merge_json_values_non_objects() {
        let base = json!("original");
        let update = json!("updated");

        let merged = InternalMemoryCollection::merge_json_values(base, update);
        assert_eq!(merged, json!("updated"));

        let base = json!(42);
        let update = json!(100);

        let merged = InternalMemoryCollection::merge_json_values(base, update);
        assert_eq!(merged, json!(100));
    }

    #[test]
    fn test_id_manager_integration() {
        let mut collection = create_test_collection();

        // Add items and verify sequential IDs
        let item1 = collection.add(json!({"name": "Item 1"})).unwrap();
        assert_eq!(item1.get("id").unwrap(), "1");

        let item2 = collection.add(json!({"name": "Item 2"})).unwrap();
        assert_eq!(item2.get("id").unwrap(), "2");

        let item3 = collection.add(json!({"name": "Item 3"})).unwrap();
        assert_eq!(item3.get("id").unwrap(), "3");
    }

    #[test]
    fn test_custom_id_key() {
        let mut collection = InternalMemoryCollection::new(
            CollectionConfig::int("customId", "custom_collection")
        );

        let item = collection.add(json!({"name": "Test Item"})).unwrap();
        assert_eq!(item.get("customId").unwrap(), "1");
        assert!(item.get("id").is_none()); // Should not have regular "id" field

        // Test retrieval
        let retrieved = collection.get("1").unwrap();
        assert_eq!(retrieved.get("customId").unwrap(), "1");
    }

    // Tests for load_from_file method
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_from_file_valid_json_array() {
        let mut collection = create_test_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_data.json");

        // Create a test JSON file with valid array data
        let test_data = json!([
            {"id": 1, "name": "Item 1", "description": "First item"},
            {"id": 2, "name": "Item 2", "description": "Second item"},
            {"id": 3, "name": "Item 3", "description": "Third item"}
        ]);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        // Load data from file
        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Loaded 3 initial items"));
        assert_eq!(collection.count(), 3);

        // Verify the data was loaded correctly
        assert!(collection.exists("1"));
        assert!(collection.exists("2"));
        assert!(collection.exists("3"));

        let item1 = collection.get("1").unwrap();
        assert_eq!(item1.get("name").unwrap(), "Item 1");
        assert_eq!(item1.get("description").unwrap(), "First item");
    }

    #[test]
    fn test_load_from_file_empty_array() {
        let mut collection = create_test_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty_array.json");

        // Create a test JSON file with empty array
        let test_data = json!([]);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        // Load data from file
        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Loaded 0 initial items"));
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_load_from_file_with_uuid_collection() {
        let mut collection = create_uuid_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("uuid_data.json");

        // Create a test JSON file with UUID data
        let test_data = json!([
            {"id": "uuid-1", "name": "Item 1"},
            {"id": "uuid-2", "name": "Item 2"}
        ]);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        // Load data from file
        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Loaded 2 initial items"));
        assert_eq!(collection.count(), 2);

        assert!(collection.exists("uuid-1"));
        assert!(collection.exists("uuid-2"));
    }

    #[test]
    fn test_load_from_file_with_mixed_id_types() {
        let mut collection = create_none_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("mixed_data.json");

        // Create a test JSON file with mixed ID types
        let test_data = json!([
            {"id": "string-id", "name": "Item 1"},
            {"id": 42, "name": "Item 2"},
            {"name": "Item 3"} // This should be skipped (no ID)
        ]);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        // Load data from file
        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Loaded 2 initial items"));
        assert_eq!(collection.count(), 2);

        assert!(collection.exists("string-id"));
        assert!(collection.exists("42"));
    }

    #[test]
    fn test_load_from_file_nonexistent_file() {
        let mut collection = create_test_collection();
        let nonexistent_path = std::ffi::OsString::from("/path/that/does/not/exist.json");

        let result = collection.load_from_file(&nonexistent_path);

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Could not read file"));
        assert!(error_msg.contains("skipping initial data load"));
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_load_from_file_invalid_json() {
        let mut collection = create_test_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.json");

        // Create a file with invalid JSON
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"{ invalid json content }").unwrap();

        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("does not contain valid JSON"));
        assert!(error_msg.contains("skipping initial data load"));
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_load_from_file_json_object_not_array() {
        let mut collection = create_test_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("object.json");

        // Create a JSON file with an object instead of array
        let test_data = json!({"id": 1, "name": "Single Item"});

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("does not contain a JSON array"));
        assert!(error_msg.contains("skipping initial data load"));
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_load_from_file_json_primitive_not_array() {
        let mut collection = create_test_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("primitive.json");

        // Create a JSON file with a primitive value
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"\"just a string\"").unwrap();

        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not contain a JSON array"));
        assert_eq!(collection.count(), 0);
    }

    #[test]
    fn test_load_from_file_updates_id_manager() {
        let mut collection = create_test_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("id_update_test.json");

        // Create data with high ID values
        let test_data = json!([
            {"id": 10, "name": "Item 1"},
            {"id": 15, "name": "Item 2"},
            {"id": 5, "name": "Item 3"}
        ]);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        // Load data from file
        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());
        assert!(result.is_ok());

        // Add a new item - should get ID 16 (max + 1)
        let new_item = collection.add(json!({"name": "New Item"})).unwrap();
        assert_eq!(new_item.get("id").unwrap(), "16");
    }

    #[test]
    fn test_load_from_file_large_dataset() {
        let mut collection = create_test_collection();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large_dataset.json");

        // Create a large dataset
        let mut items = Vec::new();
        for i in 1..=1000 {
            items.push(json!({
                "id": i,
                "name": format!("Item {}", i),
                "value": i * 10
            }));
        }
        let test_data = json!(items);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        // Load data from file
        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Loaded 1000 initial items"));
        assert_eq!(collection.count(), 1000);

        // Verify some random items
        assert!(collection.exists("1"));
        assert!(collection.exists("500"));
        assert!(collection.exists("1000"));

        let item_500 = collection.get("500").unwrap();
        assert_eq!(item_500.get("name").unwrap(), "Item 500");
        assert_eq!(item_500.get("value").unwrap(), 5000);
    }

    #[test]
    fn test_load_from_file_with_existing_data() {
        let mut collection = create_test_collection();

        // Add some existing data
        collection.add(json!({"name": "Existing Item 1"}));
        collection.add(json!({"name": "Existing Item 2"}));
        assert_eq!(collection.count(), 2);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("additional_data.json");

        // Create additional data
        let test_data = json!([
            {"id": 10, "name": "Loaded Item 1"},
            {"id": 11, "name": "Loaded Item 2"}
        ]);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        // Load additional data from file
        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Loaded 2 initial items"));
        assert_eq!(collection.count(), 4); // 2 existing + 2 loaded

        // Verify all data exists
        assert!(collection.exists("1")); // Existing
        assert!(collection.exists("2")); // Existing
        assert!(collection.exists("10")); // Loaded
        assert!(collection.exists("11")); // Loaded
    }

    #[test]
    fn test_load_from_file_custom_id_key() {
        let mut collection = InternalMemoryCollection::new(
            CollectionConfig::int("customId", "custom_collection")
        );

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("custom_id_data.json");

        // Create data with custom ID key
        let test_data = json!([
            {"customId": 1, "name": "Item 1"},
            {"customId": 2, "name": "Item 2"}
        ]);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(test_data.to_string().as_bytes()).unwrap();

        // Load data from file
        let result = collection.load_from_file(&file_path.as_os_str().to_os_string());

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Loaded 2 initial items"));
        assert_eq!(collection.count(), 2);

        assert!(collection.exists("1"));
        assert!(collection.exists("2"));

        let item1 = collection.get("1").unwrap();
        assert_eq!(item1.get("customId").unwrap(), 1);
        assert_eq!(item1.get("name").unwrap(), "Item 1");
    }

    // Tests for get_from_criteria method
    #[test]
    fn test_get_from_criteria_equal() {
        let mut collection = create_test_collection();

        // Add test data
        collection.add(json!({"name": "Alice", "age": 25, "city": "New York"}));
        collection.add(json!({"name": "Bob", "age": 30, "city": "Boston"}));
        collection.add(json!({"name": "Alice", "age": 28, "city": "Seattle"}));
        collection.add(json!({"name": "Charlie", "age": 25, "city": "New York"}));

        // Test equal comparison for string
        let constraint = Constraint::try_new(
            "name".to_string(),
            Comparer::Equal,
            Some(json!("Alice"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2);

        for result in &results {
            assert_eq!(result.get("name").unwrap(), "Alice");
        }

        // Test equal comparison for number
        let constraint = Constraint::try_new(
            "age".to_string(),
            Comparer::Equal,
            Some(json!(25))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2);

        for result in &results {
            assert_eq!(result.get("age").unwrap(), 25);
        }
    }

    #[test]
    fn test_get_from_criteria_different() {
        let mut collection = create_test_collection();

        // Add test data
        collection.add(json!({"name": "Alice", "age": 25, "active": true}));
        collection.add(json!({"name": "Bob", "age": 30, "active": false}));
        collection.add(json!({"name": "Charlie", "age": 25, "active": true}));

        // Test different comparison for string
        let constraint = Constraint::try_new(
            "name".to_string(),
            Comparer::Different,
            Some(json!("Alice"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2);

        for result in &results {
            assert_ne!(result.get("name").unwrap(), "Alice");
        }

        // Test different comparison for boolean
        let constraint = Constraint::try_new(
            "active".to_string(),
            Comparer::Different,
            Some(json!(true))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name").unwrap(), "Bob");
    }

    #[test]
    fn test_get_from_criteria_numeric_comparisons() {
        let mut collection = create_test_collection();

        // Add test data with different ages and scores
        collection.add(json!({"name": "Alice", "age": 25, "score": 85.5}));
        collection.add(json!({"name": "Bob", "age": 30, "score": 92.0}));
        collection.add(json!({"name": "Charlie", "age": 35, "score": 78.5}));
        collection.add(json!({"name": "David", "age": 20, "score": 95.0}));

        // Test greater than
        let constraint = Constraint::try_new(
            "age".to_string(),
            Comparer::GreaterThan,
            Some(json!(25))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Bob (30) and Charlie (35)

        let ages: Vec<i64> = results.iter()
            .map(|r| r.get("age").unwrap().as_i64().unwrap())
            .collect();
        assert!(ages.contains(&30));
        assert!(ages.contains(&35));

        // Test greater than or equal
        let constraint = Constraint::try_new(
            "age".to_string(),
            Comparer::GreaterThanOrEqual,
            Some(json!(30))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Bob (30) and Charlie (35)

        // Test less than
        let constraint = Constraint::try_new(
            "score".to_string(),
            Comparer::LessThan,
            Some(json!(90.0))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Alice (85.5) and Charlie (78.5)

        // Test less than or equal
        let constraint = Constraint::try_new(
            "score".to_string(),
            Comparer::LessThanOrEqual,
            Some(json!(85.5))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Alice (85.5) and Charlie (78.5)
    }

    #[test]
    fn test_get_from_criteria_like_patterns() {
        let mut collection = create_test_collection();

        // Add test data with email addresses
        collection.add(json!({"name": "Alice", "email": "alice@gmail.com"}));
        collection.add(json!({"name": "Bob", "email": "bob@company.com"}));
        collection.add(json!({"name": "Charlie", "email": "charlie@gmail.com"}));
        collection.add(json!({"name": "David", "email": "david@yahoo.com"}));

        // Test LIKE with % wildcard
        let constraint = Constraint::try_new(
            "email".to_string(),
            Comparer::Like,
            Some(json!("%@gmail.com"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Alice and Charlie

        let names: Vec<&str> = results.iter()
            .map(|r| r.get("name").unwrap().as_str().unwrap())
            .collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Charlie"));

        // Test LIKE with _ wildcard
        let constraint = Constraint::try_new(
            "name".to_string(),
            Comparer::Like,
            Some(json!("B_b"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name").unwrap(), "Bob");

        // Test complex pattern
        let constraint = Constraint::try_new(
            "email".to_string(),
            Comparer::Like,
            Some(json!("%@%.com"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 4); // All emails end with .com
    }

    #[test]
    fn test_get_from_criteria_null_checks() {
        let mut collection = create_test_collection();

        // Add test data with some null values
        collection.add(json!({"name": "Alice", "phone": "123-456-7890", "notes": null}));
        collection.add(json!({"name": "Bob", "phone": null, "notes": "Important client"}));
        collection.add(json!({"name": "Charlie", "phone": "987-654-3210", "notes": null}));

        // Test IS NULL
        let constraint = Constraint::try_new(
            "phone".to_string(),
            Comparer::IsNull,
            None
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name").unwrap(), "Bob");

        // Test IS NOT NULL
        let constraint = Constraint::try_new(
            "phone".to_string(),
            Comparer::IsNotNull,
            None
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Alice and Charlie

        let names: Vec<&str> = results.iter()
            .map(|r| r.get("name").unwrap().as_str().unwrap())
            .collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Charlie"));

        // Test IS NULL for notes
        let constraint = Constraint::try_new(
            "notes".to_string(),
            Comparer::IsNull,
            None
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Alice and Charlie
    }

    #[test]
    fn test_get_from_criteria_no_matches() {
        let mut collection = create_test_collection();

        // Add test data
        collection.add(json!({"name": "Alice", "age": 25}));
        collection.add(json!({"name": "Bob", "age": 30}));

        // Test with criteria that matches nothing
        let constraint = Constraint::try_new(
            "name".to_string(),
            Comparer::Equal,
            Some(json!("NonExistent"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert!(results.is_empty());

        // Test with field that doesn't exist
        let constraint = Constraint::try_new(
            "salary".to_string(),
            Comparer::GreaterThan,
            Some(json!(50000))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_from_criteria_empty_collection() {
        let collection = create_test_collection();

        let constraint = Constraint::try_new(
            "name".to_string(),
            Comparer::Equal,
            Some(json!("Alice"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_from_criteria_with_non_object_values() {
        let mut collection = InternalMemoryCollection::new(
            CollectionConfig::none("id", "test_collection")
        );

        // Manually insert some non-object values (this shouldn't happen in normal usage)
        collection.collection.insert("1".to_string(), json!("string_value"));
        collection.collection.insert("2".to_string(), json!(42));
        collection.collection.insert("3".to_string(), json!({"name": "Alice", "age": 25}));

        let constraint = Constraint::try_new(
            "name".to_string(),
            Comparer::Equal,
            Some(json!("Alice"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 1); // Only the object should match
        assert_eq!(results[0].get("name").unwrap(), "Alice");
    }

    #[test]
    fn test_get_from_criteria_complex_data() {
        let mut collection = create_test_collection();

        // Add complex test data
        collection.add(json!({
            "name": "Alice",
            "age": 25,
            "department": "Engineering",
            "salary": 75000.50,
            "active": true,
            "skills": ["Rust", "JavaScript", "Python"],
            "address": {
                "city": "New York",
                "state": "NY"
            }
        }));

        collection.add(json!({
            "name": "Bob",
            "age": 30,
            "department": "Marketing",
            "salary": 65000.00,
            "active": false,
            "skills": ["Marketing", "Analytics"],
            "address": {
                "city": "Boston",
                "state": "MA"
            }
        }));

        collection.add(json!({
            "name": "Charlie",
            "age": 35,
            "department": "Engineering",
            "salary": 85000.75,
            "active": true,
            "skills": ["Java", "Python", "SQL"],
            "address": {
                "city": "Seattle",
                "state": "WA"
            }
        }));

        // Test filtering by department
        let constraint = Constraint::try_new(
            "department".to_string(),
            Comparer::Equal,
            Some(json!("Engineering"))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Alice and Charlie

        // Test filtering by salary range
        let constraint = Constraint::try_new(
            "salary".to_string(),
            Comparer::GreaterThan,
            Some(json!(70000.0))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Alice and Charlie

        // Test filtering by active status
        let constraint = Constraint::try_new(
            "active".to_string(),
            Comparer::Equal,
            Some(json!(true))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2); // Alice and Charlie

        let names: Vec<&str> = results.iter()
            .map(|r| r.get("name").unwrap().as_str().unwrap())
            .collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Charlie"));
    }

    #[test]
    fn test_get_from_criteria_edge_cases() {
        let mut collection = create_test_collection();

        // Add data with edge case values
        collection.add(json!({"name": "", "score": 0, "flag": false}));
        collection.add(json!({"name": " ", "score": 0.0, "flag": true}));
        collection.add(json!({"name": "Test", "score": -1, "flag": false}));

        // Test empty string
        let constraint = Constraint::try_new(
            "name".to_string(),
            Comparer::Equal,
            Some(json!(""))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 1);

        // Test zero values - note that JSON treats 0 and 0.0 differently
        let constraint = Constraint::try_new(
            "score".to_string(),
            Comparer::Equal,
            Some(json!(0))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 1); // Only the integer 0 should match

        // Test with 0.0 specifically
        let constraint = Constraint::try_new(
            "score".to_string(),
            Comparer::Equal,
            Some(json!(0.0))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 1); // Only the float 0.0 should match

        // Test false boolean
        let constraint = Constraint::try_new(
            "flag".to_string(),
            Comparer::Equal,
            Some(json!(false))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 2);

        // Test negative numbers
        let constraint = Constraint::try_new(
            "score".to_string(),
            Comparer::LessThan,
            Some(json!(0))
        ).unwrap();

        let results = collection.get_from_constraint(&constraint);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name").unwrap(), "Test");
    }
}
