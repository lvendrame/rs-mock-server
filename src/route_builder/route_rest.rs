use std::{ffi::OsString};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{app::App, handlers::{create_delete, create_full_update, create_get_all, create_get_item, create_insert, create_partial_update, load_initial_data}, id_manager::IdType, in_memory_collection::InMemoryCollection, route_builder::{route_params::RouteParams, PrintRoute, Route, RouteGenerator}};

static RE_FILE_REST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\$)?rest(\{(.+)\})?$").unwrap()
});

const ELEMENT_IS_PROTECTED: usize = 1;
const ELEMENT_DESCRIPTOR: usize = 3;

#[derive(Debug, Clone, PartialEq)]
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

    pub fn try_parse(route_params: RouteParams) -> Route {
        if let Some(captures) = RE_FILE_REST.captures(&route_params.file_stem) {
            let is_protected = route_params.is_protected || captures.get(ELEMENT_IS_PROTECTED).is_some();
            let descriptor = if let Some(pattern) = captures.get(ELEMENT_DESCRIPTOR) {
                pattern.as_str()
            } else {
                "id:uuid"
            };

            let (id_key, id_type) = Self::get_rest_options(descriptor);

            let route_rest = Self {
                path: route_params.file_path,
                route: route_params.full_route,
                id_key: id_key.to_string(),
                id_type,
                is_protected,
            };

            return Route::Rest(route_rest);
        }

        Route::None
    }
}

impl RouteGenerator for RouteRest {
    fn make_routes(&self, app: &mut App) {
        let in_memory_collection = InMemoryCollection::new(self.id_type, self.id_key.clone());
        let collection = in_memory_collection.into_protected();

        let route_path = self.path.to_str().unwrap();

        load_initial_data(&self.path, &collection);

        let id_route = format!("{}/{{{}}}", route_path, self.id_key);

        // Build REST routes for CRUD operations
        create_get_all(app, route_path, &collection, self.is_protected);

        create_insert(app, route_path, &collection, self.is_protected);

        create_get_item(app, &collection, &id_route, self.is_protected);

        create_full_update(app, &collection, &id_route, self.is_protected);

        create_partial_update(app, &collection, &id_route, self.is_protected);

        create_delete(app, collection, &id_route, self.is_protected);
    }
}

impl PrintRoute for RouteRest {
    fn println(&self) {
        println!("✔️ Built REST routes for {}", self.route);
    }
}