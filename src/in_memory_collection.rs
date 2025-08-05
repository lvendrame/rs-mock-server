use std::{collections::HashMap, sync::{Arc, Mutex}};
use serde_json::Value;

use crate::id_manager::{IdManager, IdType, IdValue};

pub type ProtectedMemCollection = Arc<Mutex<InMemoryCollection>>;

pub struct InMemoryCollection {
    db: HashMap<String, Value>,
    id_manager: IdManager,
    id_key: String,
    pub name: Option<String>,
}

impl InMemoryCollection {
    pub fn new(id_type: IdType, id_key: String, name: Option<String>) -> Self {
        let db: HashMap<String, Value> = HashMap::new();
        let id_manager = IdManager::new(id_type);
        Self {
            db,
            id_manager,
            id_key,
            name
        }
    }

    pub fn into_protected(self) -> ProtectedMemCollection {
        Arc::new(Mutex::new(self))
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

        let mut item = item;
        let id_string = if let Some(id_value) = next_id {
            // Convert IdValue to string and add it to the item
            let id_string = id_value.to_string();

            // Add the ID to the item using the configured id_key
            if let Value::Object(ref mut map) = item {
                map.insert(self.id_key.clone(), Value::String(id_string.clone()));
            }
            Some(id_string)
        } else if let Some(Value::String(id_string)) = item.get(self.id_key.clone()){
            Some(id_string.clone())
        } else if let Some(Value::Number(id_number)) = item.get(self.id_key.clone()){
            Some(id_number.to_string())
        }else {
            None
        };

        if let Some(id_string) = id_string {
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
                        IdType::None => match item.get(self.id_key.clone()) {
                            Some(Value::String(id_string)) => Some(id_string.clone()),
                            Some(Value::Number(id_number)) => Some(id_number.to_string()),
                            _ => None
                        }
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
                if self.id_manager.set_current(IdValue::Int(value)).is_err() {
                    println!("Error to set the value {} to {} collection Id", value, self.name.clone().unwrap_or("{{unknown}}".to_string()));
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_collection() -> InMemoryCollection {
        InMemoryCollection::new(IdType::Int, "id".to_string(), Some("test_collection".to_string()))
    }

    fn create_uuid_collection() -> InMemoryCollection {
        InMemoryCollection::new(IdType::Uuid, "id".to_string(), Some("uuid_collection".to_string()))
    }

    fn create_none_collection() -> InMemoryCollection {
        InMemoryCollection::new(IdType::None, "id".to_string(), Some("none_collection".to_string()))
    }

    #[test]
    fn test_new_collection() {
        let collection = create_test_collection();
        assert_eq!(collection.count(), 0);
        assert_eq!(collection.id_key, "id");
        assert_eq!(collection.name, Some("test_collection".to_string()));
    }

    #[test]
    fn test_into_protected() {
        let collection = create_test_collection();
        let protected = collection.into_protected();

        let guard = protected.lock().unwrap();
        assert_eq!(guard.count(), 0);
        assert_eq!(guard.name, Some("test_collection".to_string()));
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

        let merged = InMemoryCollection::merge_json_values(base, update);

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

        let merged = InMemoryCollection::merge_json_values(base, update);
        assert_eq!(merged, json!("updated"));

        let base = json!(42);
        let update = json!(100);

        let merged = InMemoryCollection::merge_json_values(base, update);
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
        let mut collection = InMemoryCollection::new(
            IdType::Int,
            "customId".to_string(),
            Some("custom_collection".to_string())
        );

        let item = collection.add(json!({"name": "Test Item"})).unwrap();
        assert_eq!(item.get("customId").unwrap(), "1");
        assert!(item.get("id").is_none()); // Should not have regular "id" field

        // Test retrieval
        let retrieved = collection.get("1").unwrap();
        assert_eq!(retrieved.get("customId").unwrap(), "1");
    }
}