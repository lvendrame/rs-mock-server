use crate::memory_db::IdType;

#[derive(Debug, Clone)]
pub struct CollectionConfig {
    pub id_type: IdType,
    pub id_key: String,
    pub name: String,
}

impl CollectionConfig {
    pub fn new() -> Self {
        Self {
            id_type: IdType::Uuid,
            id_key: "id".to_string(),
            name: "{unknown}".to_string(),
        }
    }

    pub fn from(id_type: IdType, id_key: &str, name: &str) -> Self {
        Self {
            id_type,
            id_key: id_key.to_string(),
            name: name.to_string(),
        }
    }

    pub fn int(id_key: &str, name: &str) -> Self {
        Self {
            id_type: IdType::Int,
            id_key: id_key.to_string(),
            name: name.to_string(),
        }
    }
    pub fn uuid(id_key: &str, name: &str) -> Self {
        Self {
            id_type: IdType::Uuid,
            id_key: id_key.to_string(),
            name: name.to_string(),
        }
    }
    pub fn none(id_key: &str, name: &str) -> Self {
        Self {
            id_type: IdType::None,
            id_key: id_key.to_string(),
            name: name.to_string(),
        }
    }
}
