use std::fmt::Display;

use uuid::Uuid;

pub enum IdType {
    Uuid,
    Int,
}

#[derive(Clone)]
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

    pub fn set_current(&mut self, value: IdValue) {
        self.current = Some(value);
    }
}

impl Iterator for IdManager{
    type Item = IdValue;
    fn next(&mut self) -> Option<Self::Item> {
        let item = match &self.current {
            Some(IdValue::Int(id)) => IdValue::Int(id + 1),
            Some(IdValue::Uuid(_)) => IdValue::Uuid(Uuid::new_v4().to_string()),
            None => match self.id_type {
                IdType::Int => IdValue::Int(1),
                IdType::Uuid => IdValue::Uuid(Uuid::new_v4().to_string()),
            }
        };

        self.current = Some(item.clone());
        Some(item)
    }
}