use std::ffi::OsString;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{id_manager::IdType, route_builder::PrintRoute};

static RE_FILE_REST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?rest(\{(.+)\})?$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;
const ELEMENT_DESCRIPTOR: usize = 3;

pub struct  RouteRest {
    pub path: OsString,
    pub route: String,
    pub id_key: String,
    pub id_type: IdType,
    pub is_protected: bool,
}

impl RouteRest {
    fn get_rest_options(descriptor: &str) -> (&str, IdType) {
        let parts: Vec<&str> = descriptor.split(':').collect();

        if parts.len() == 1 {
            // Single value like "uuid", "int", "id", "_id"
            let part = parts[0];
            match part {
                "uuid" => ("id", IdType::Uuid),
                "int" => ("id", IdType::Int),
                id_key => (id_key, IdType::Uuid), // Default fallback
            }
        } else if parts.len() == 2 {
            // Key:type format like "id:uuid", "_id:int"
            let id_key = parts[0];
            let type_str = parts[1];
            let id_type = match type_str {
                "uuid" => IdType::Uuid,
                "int" => IdType::Int,
                _ => IdType::Uuid, // Default to UUID
            };
            (id_key, id_type)
        } else {
            // Invalid format, return defaults
            ("id", IdType::Uuid)
        }
    }

    pub fn try_parse(parent_route: &str, file_name: String, file_path: OsString, is_protected: bool) -> Option<Self> {
        let file_stem = file_name.split('.').next().unwrap_or("");

        if let Some(captures) = RE_FILE_REST.captures(file_stem) {
            let is_protected = is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let descriptor = if let Some(pattern) = captures.get(ELEMENT_DESCRIPTOR) {
                pattern.as_str()
            } else {
                "id:uuid"
            };

            let (id_key, id_type) = Self::get_rest_options(descriptor);
            let route = if parent_route.is_empty() { "/" } else { parent_route };

            let route_rest = Self {
                path: file_path,
                route: route.to_string(),
                id_key: id_key.to_string(),
                id_type,
                is_protected,
            };

            return Some(route_rest);
        }

        None
    }
}

impl PrintRoute for RouteRest {
    fn println(&self) {
        println!("✔️ Built REST routes for {}", self.route);
    }
}