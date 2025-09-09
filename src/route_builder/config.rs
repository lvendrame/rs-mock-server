//! Configuration module for the mock server, defining structures for loading and storing configuration from TOML files.

use std::{collections::HashMap, fs::{self, DirEntry}};

use fosk::IdType;
use serde::{Deserialize, Serialize};
use toml::de::Error as DeserializeError;

/// Represents the combined configuration for the mock server.
///
/// This configuration can be loaded from TOML and applies settings
/// at server level, default route level, collection defaults,
/// authentication, and upload behavior.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Server-level configuration options.
    pub server: Option<ServerConfig>,
    /// Default route configuration options.
    pub route: Option<RouteConfig>,
    /// Default Fosk collection configuration options.
    pub collection: Option<CollectionConfig>,
    /// Authentication configuration options.
    pub auth: Option<AuthConfig>,
    /// Upload configuration options.
    pub upload: Option<UploadConfig>,
}

/// Server configuration settings such as port, static folder, and CORS.
///
/// These settings apply globally to the mock server.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Port number the server listens on.
    pub port: Option<u16>,
    /// Filesystem path to serve static files from.
    pub folder: Option<String>,
    /// Enable or disable Cross-Origin Resource Sharing.
    pub enable_cors: Option<bool>,
    /// Allowed origin for CORS requests.
    pub allowed_origin: Option<String>,
}

/// Route-specific configuration settings.
///
/// Allows overriding default delay, remapping paths,
/// and protection for individual routes.
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Artificial delay (in milliseconds) before responding.
    pub delay: Option<u16>,
    /// Remapped path for the route.
    pub remap: Option<String>,
    /// Protect the route (e.g., require authentication).
    pub protect: Option<bool>,
}

/// Configuration for Fosk collections.
///
/// Defines naming and identifier handling for Fosk collections.
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Name of the Fosk collection.
    pub name: Option<String>,
    /// Field name to use as the identifier key in the Fosk collection.
    pub id_key: Option<String>,
    /// Strategy for generating or interpreting Fosk collection identifiers.
    pub id_type: Option<IdType>,
}

/// Authentication-related configuration.
///
/// Includes user credentials, cookie settings, JWT secret,
/// and routes for login, logout, and user management.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Field name for usernames in auth payloads.
    pub username_field: Option<String>,
    /// Field name for passwords in auth payloads.
    pub password_field: Option<String>,
    /// Field name specifying user roles.
    pub roles_field: Option<String>,
    /// Name of the authentication cookie.
    pub cookie_name: Option<String>,
    /// Whether to encrypt passwords before storing.
    pub encrypt_password: Option<bool>,
    /// Secret key for signing JWT tokens.
    pub jwt_secret: Option<String>,
    /// Fosk collection configuration for storing tokens.
    pub token_collection: Option<CollectionConfig>,
    /// Fosk collection configuration for storing user data.
    pub user_collection: Option<CollectionConfig>,
    /// Route path for user login.
    pub login_route: Option<String>,
    /// Route path for user logout.
    pub logout_route: Option<String>,
    /// Route path for user management.
    pub users_route: Option<String>,
}

/// File upload configuration settings.
///
/// Defines routes and behavior for uploading, downloading,
/// and listing files, including temporary storage options.
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadConfig {
    /// Route path for handling file uploads.
    pub upload_route: Option<String>,
    /// Route path for file downloads.
    pub download_route: Option<String>,
    /// Route path for listing files.
    pub list_files_route: Option<String>,
    /// Use temporary storage for uploads.
    pub temporary: Option<bool>,
}

impl TryFrom<&str> for Config {
    type Error = DeserializeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        toml::from_str(value)
    }
}

impl TryFrom<&DirEntry> for Config {
    type Error = String;

    fn try_from(value: &DirEntry) -> Result<Self, Self::Error> {
        let content = fs::read_to_string(value.path())
            .map_err(|e| e.to_string())?;

        Config::try_from(content.as_str())
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug, Default)]
pub struct ConfigStore {
    map_configs: HashMap<String, Config>,
}

impl ConfigStore {
    pub fn try_from_dir(dir_path: &str) -> Result<Self,std::io::Error> {
        let mut store = Self::default();
        fs::read_dir( dir_path)?
            .filter_map(Result::ok)
            .filter(|file| file.path().ends_with(".toml"))
            .for_each(|file| {
                let key = file.path().as_path().file_stem().unwrap().to_string_lossy().to_ascii_lowercase();
                match Config::try_from(&file) {
                    Ok(config) => { store.map_configs.insert(key, config); },
                    Err(err) =>
                        println!("Unable to load the config file {:?} due the error {}.", file.file_name(), err),
                }
            });

        Ok(store)
    }
}
