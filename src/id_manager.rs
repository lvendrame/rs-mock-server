use std::fmt::Display;

use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum IdType {
    #[default]
    Uuid,
    Int,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdValue {
    Uuid(String),
    Int(u64),
}

impl Display for IdValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = match self {
            IdValue::Int(id) => id.to_string(),
            IdValue::Uuid(uuid) => uuid.clone(),
        };

        f.write_str(&id)
    }
}

pub struct IdManager {
    pub id_type: IdType,
    pub current: Option<IdValue>,
}

impl IdManager {
    pub fn new(id_type: IdType) -> Self {
        Self {
            id_type,
            current: None
        }
    }

    pub fn set_current(&mut self, value: IdValue) -> Result<(), String> {
        match (&self.id_type, &value) {
            (IdType::Int, IdValue::Int(_)) => {
                self.current = Some(value);
                Ok(())
            },
            (IdType::Uuid, IdValue::Uuid(_)) => {
                self.current = Some(value);
                Ok(())
            },
            (IdType::None, _) => {
                Err("Cannot set current value for IdType::None".to_string())
            },
            (IdType::Int, IdValue::Uuid(_)) => {
                Err("Cannot set UUID value for Int IdManager".to_string())
            },
            (IdType::Uuid, IdValue::Int(_)) => {
                Err("Cannot set Int value for UUID IdManager".to_string())
            },
        }
    }

}

impl Iterator for IdManager{
    type Item = IdValue;
    fn next(&mut self) -> Option<Self::Item> {
        let item = match &self.current {
            Some(IdValue::Int(id)) => match *id {
                u64::MAX => IdValue::Int(0),
                _ => IdValue::Int(id + 1)
            },
            Some(IdValue::Uuid(_)) => IdValue::Uuid(Uuid::new_v4().to_string()),
            None => match self.id_type {
                IdType::Int => IdValue::Int(1),
                IdType::Uuid => IdValue::Uuid(Uuid::new_v4().to_string()),
                IdType::None => return None,
            }
        };

        self.current = Some(item.clone());
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_type_default() {
        let id_type = IdType::default();
        assert_eq!(id_type, IdType::Uuid);
    }

    #[test]
    fn test_id_type_equality() {
        assert_eq!(IdType::Uuid, IdType::Uuid);
        assert_eq!(IdType::Int, IdType::Int);
        assert_eq!(IdType::None, IdType::None);
        assert_ne!(IdType::Uuid, IdType::Int);
        assert_ne!(IdType::Int, IdType::None);
        assert_ne!(IdType::Uuid, IdType::None);
    }

    #[test]
    fn test_id_value_display_int() {
        let id_value = IdValue::Int(42);
        assert_eq!(id_value.to_string(), "42");
        assert_eq!(format!("{}", id_value), "42");
    }

    #[test]
    fn test_id_value_display_uuid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id_value = IdValue::Uuid(uuid_str.to_string());
        assert_eq!(id_value.to_string(), uuid_str);
        assert_eq!(format!("{}", id_value), uuid_str);
    }

    #[test]
    fn test_id_value_equality() {
        let int1 = IdValue::Int(42);
        let int2 = IdValue::Int(42);
        let int3 = IdValue::Int(43);

        let uuid1 = IdValue::Uuid("test-uuid".to_string());
        let uuid2 = IdValue::Uuid("test-uuid".to_string());
        let uuid3 = IdValue::Uuid("other-uuid".to_string());

        assert_eq!(int1, int2);
        assert_ne!(int1, int3);
        assert_eq!(uuid1, uuid2);
        assert_ne!(uuid1, uuid3);
        assert_ne!(int1, uuid1);
    }

    #[test]
    fn test_id_value_clone() {
        let int_value = IdValue::Int(100);
        let cloned_int = int_value.clone();
        assert_eq!(int_value, cloned_int);

        let uuid_value = IdValue::Uuid("test-uuid".to_string());
        let cloned_uuid = uuid_value.clone();
        assert_eq!(uuid_value, cloned_uuid);
    }

    #[test]
    fn test_id_manager_new_uuid() {
        let manager = IdManager::new(IdType::Uuid);
        assert_eq!(manager.id_type, IdType::Uuid);
        assert_eq!(manager.current, None);
    }

    #[test]
    fn test_id_manager_new_int() {
        let manager = IdManager::new(IdType::Int);
        assert_eq!(manager.id_type, IdType::Int);
        assert_eq!(manager.current, None);
    }

    #[test]
    fn test_id_manager_new_none() {
        let manager = IdManager::new(IdType::None);
        assert_eq!(manager.id_type, IdType::None);
        assert_eq!(manager.current, None);
    }

    #[test]
    fn test_id_manager_set_current_int() {
        let mut manager = IdManager::new(IdType::Int);
        let id_value = IdValue::Int(42);
        let result = manager.set_current(id_value.clone());
        assert!(result.is_ok());
        assert_eq!(manager.current, Some(id_value));
    }

    #[test]
    fn test_id_manager_set_current_uuid() {
        let mut manager = IdManager::new(IdType::Uuid);
        let id_value = IdValue::Uuid("test-uuid".to_string());
        let result = manager.set_current(id_value.clone());
        assert!(result.is_ok());
        assert_eq!(manager.current, Some(id_value));
    }

    #[test]
    fn test_id_manager_iterator_int_from_none() {
        let mut manager = IdManager::new(IdType::Int);

        // First call should return 1
        let first = manager.next();
        assert_eq!(first, Some(IdValue::Int(1)));
        assert_eq!(manager.current, Some(IdValue::Int(1)));

        // Second call should return 2
        let second = manager.next();
        assert_eq!(second, Some(IdValue::Int(2)));
        assert_eq!(manager.current, Some(IdValue::Int(2)));

        // Third call should return 3
        let third = manager.next();
        assert_eq!(third, Some(IdValue::Int(3)));
        assert_eq!(manager.current, Some(IdValue::Int(3)));
    }

    #[test]
    fn test_id_manager_iterator_int_from_set_value() {
        let mut manager = IdManager::new(IdType::Int);
        manager.set_current(IdValue::Int(10)).unwrap();

        // Should continue from the set value
        let next = manager.next();
        assert_eq!(next, Some(IdValue::Int(11)));
        assert_eq!(manager.current, Some(IdValue::Int(11)));

        let next = manager.next();
        assert_eq!(next, Some(IdValue::Int(12)));
        assert_eq!(manager.current, Some(IdValue::Int(12)));
    }

    #[test]
    fn test_id_manager_iterator_uuid_from_none() {
        let mut manager = IdManager::new(IdType::Uuid);

        // First call should return a UUID
        let first = manager.next();
        assert!(first.is_some());
        if let Some(IdValue::Uuid(uuid)) = &first {
            assert!(!uuid.is_empty());
            assert!(uuid.len() > 10); // UUIDs are typically 36 characters
        } else {
            panic!("Expected UUID value");
        }

        // Current should be set
        assert!(manager.current.is_some());

        // Second call should return a different UUID
        let second = manager.next();
        assert!(second.is_some());
        assert_ne!(first, second); // UUIDs should be different
    }

    #[test]
    fn test_id_manager_iterator_uuid_from_set_value() {
        let mut manager = IdManager::new(IdType::Uuid);
        let initial_uuid = "initial-uuid".to_string();
        manager.set_current(IdValue::Uuid(initial_uuid.clone())).unwrap();

        // Should generate a new UUID, not increment the existing one
        let next = manager.next();
        assert!(next.is_some());
        if let Some(IdValue::Uuid(uuid)) = &next {
            assert_ne!(uuid, &initial_uuid);
            assert!(!uuid.is_empty());
        } else {
            panic!("Expected UUID value");
        }
    }

    #[test]
    fn test_id_manager_iterator_none_type() {
        let mut manager = IdManager::new(IdType::None);

        // Should always return None
        let first = manager.next();
        assert_eq!(first, None);
        assert_eq!(manager.current, None);

        let second = manager.next();
        assert_eq!(second, None);
        assert_eq!(manager.current, None);
    }

    #[test]
    fn test_id_manager_iterator_none_type_with_set_value() {
        let mut manager = IdManager::new(IdType::None);
        let result = manager.set_current(IdValue::Int(42));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot set current value for IdType::None");

        // Should always return None even after failed set_current
        let next = manager.next();
        assert_eq!(next, None);
        assert_eq!(manager.current, None);
    }

    #[test]
    fn test_id_manager_iterator_collect_int() {
        let manager = IdManager::new(IdType::Int);
        let ids: Vec<IdValue> = manager.take(5).collect();

        assert_eq!(ids.len(), 5);
        assert_eq!(ids[0], IdValue::Int(1));
        assert_eq!(ids[1], IdValue::Int(2));
        assert_eq!(ids[2], IdValue::Int(3));
        assert_eq!(ids[3], IdValue::Int(4));
        assert_eq!(ids[4], IdValue::Int(5));
    }

    #[test]
    fn test_id_manager_iterator_collect_uuid() {
        let manager = IdManager::new(IdType::Uuid);
        let ids: Vec<IdValue> = manager.take(3).collect();

        assert_eq!(ids.len(), 3);

        // All should be UUIDs and different from each other
        for id in &ids {
            if let IdValue::Uuid(uuid) = id {
                assert!(!uuid.is_empty());
            } else {
                panic!("Expected UUID value");
            }
        }

        // All UUIDs should be different
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        assert_ne!(ids[0], ids[2]);
    }

    #[test]
    fn test_id_manager_mixed_operations() {
        let mut manager = IdManager::new(IdType::Int);

        // Get first ID
        let first = manager.next().unwrap();
        assert_eq!(first, IdValue::Int(1));

        // Set a different current value
        manager.set_current(IdValue::Int(100)).unwrap();

        // Next should continue from the set value
        let next = manager.next().unwrap();
        assert_eq!(next, IdValue::Int(101));

        // Continue iterating
        let after = manager.next().unwrap();
        assert_eq!(after, IdValue::Int(102));
    }

    #[test]
    fn test_id_manager_large_int_values() {
        let mut manager = IdManager::new(IdType::Int);
        manager.set_current(IdValue::Int(u64::MAX - 2)).unwrap();

        let next = manager.next().unwrap();
        assert_eq!(next, IdValue::Int(u64::MAX - 1));

        let next = manager.next().unwrap();
        assert_eq!(next, IdValue::Int(u64::MAX));

        // This would overflow, but we test the behavior
        let next = manager.next().unwrap();
        assert_eq!(next, IdValue::Int(0)); // Wraps around due to overflow
    }

    #[test]
    fn test_id_value_debug_format() {
        let int_value = IdValue::Int(42);
        let debug_str = format!("{:?}", int_value);
        assert_eq!(debug_str, "Int(42)");

        let uuid_value = IdValue::Uuid("test-uuid".to_string());
        let debug_str = format!("{:?}", uuid_value);
        assert_eq!(debug_str, "Uuid(\"test-uuid\")");
    }

    #[test]
    fn test_id_type_debug_format() {
        assert_eq!(format!("{:?}", IdType::Uuid), "Uuid");
        assert_eq!(format!("{:?}", IdType::Int), "Int");
        assert_eq!(format!("{:?}", IdType::None), "None");
    }

    #[test]
    fn test_id_manager_type_mismatch_scenarios() {
        // Test setting UUID value on Int manager should fail
        let mut int_manager = IdManager::new(IdType::Int);
        let result = int_manager.set_current(IdValue::Uuid("test".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot set UUID value for Int IdManager");

        // Current should remain None after failed set
        assert_eq!(int_manager.current, None);

        // Next should still generate Int based on type
        let next = int_manager.next().unwrap();
        assert_eq!(next, IdValue::Int(1));

        // Test setting Int value on UUID manager should fail
        let mut uuid_manager = IdManager::new(IdType::Uuid);
        let result = uuid_manager.set_current(IdValue::Int(42));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot set Int value for UUID IdManager");

        // Current should remain None after failed set
        assert_eq!(uuid_manager.current, None);

        // Next should generate UUID based on type
        let next = uuid_manager.next().unwrap();
        if let IdValue::Uuid(_) = next {
            // Expected
        } else {
            panic!("Expected UUID value after failed Int set_current");
        }
    }

    #[test]
    fn test_id_manager_set_current_validation() {
        // Test successful cases
        let mut int_manager = IdManager::new(IdType::Int);
        assert!(int_manager.set_current(IdValue::Int(42)).is_ok());

        let mut uuid_manager = IdManager::new(IdType::Uuid);
        assert!(uuid_manager.set_current(IdValue::Uuid("test".to_string())).is_ok());

        // Test error cases
        let mut int_manager = IdManager::new(IdType::Int);
        let result = int_manager.set_current(IdValue::Uuid("test".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot set UUID value for Int IdManager");

        let mut uuid_manager = IdManager::new(IdType::Uuid);
        let result = uuid_manager.set_current(IdValue::Int(42));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot set Int value for UUID IdManager");

        // Test None type
        let mut none_manager = IdManager::new(IdType::None);
        let result = none_manager.set_current(IdValue::Int(42));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot set current value for IdType::None");

        let result = none_manager.set_current(IdValue::Uuid("test".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot set current value for IdType::None");
    }

    #[test]
    fn test_id_manager_state_after_failed_set_current() {
        let mut manager = IdManager::new(IdType::Int);

        // Set a valid value first
        manager.set_current(IdValue::Int(10)).unwrap();
        assert_eq!(manager.current, Some(IdValue::Int(10)));

        // Try to set an invalid value
        let result = manager.set_current(IdValue::Uuid("test".to_string()));
        assert!(result.is_err());

        // Current value should remain unchanged
        assert_eq!(manager.current, Some(IdValue::Int(10)));

        // Iterator should continue from the valid current value
        let next = manager.next().unwrap();
        assert_eq!(next, IdValue::Int(11));
    }
}
