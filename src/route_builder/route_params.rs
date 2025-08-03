use std::{ffi::OsString, fs::DirEntry};

#[derive(Debug, Clone)]
pub struct RouteParams {
    pub parent_route: String,
    pub full_route: String,
    pub file_name: String,
    pub file_stem: String,
    pub file_path: OsString,
    pub is_protected: bool,
    pub is_dir: bool,
}

impl RouteParams {
    pub fn new(parent_route: &str, entry: &DirEntry, is_protected: bool) -> Self {
        let parent_route = (if parent_route.is_empty() { "/" } else { parent_route }).to_string();

        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_stem = file_name.split('.').next().unwrap_or("").to_string();

        let is_dir = entry.file_type().unwrap().is_dir();

        let full_route = if is_dir {
            let end_point = file_name.replace("$", "");
            format!("{}/{}", parent_route, end_point)
        } else {
            parent_route.clone()
        };

        let file_path = entry.path().into_os_string();

        Self {
            parent_route,
            full_route,
            file_name,
            file_path,
            file_stem,
            is_protected,
            is_dir,
        }
    }
}