//! Configuration module for the mock server, defining structures for loading and storing configuration from TOML files.

use std::{
    collections::HashMap,
    fs::{self, DirEntry},
};

use fosk::IdType;
use serde::{Deserialize, Serialize};
use toml::de::Error as DeserializeError;

use crate::handlers::is_toml;

/// Represents the combined configuration for the mock server.
///
/// This configuration can be loaded from TOML and applies settings
/// at server level, default route level, collection defaults,
/// authentication, and upload behavior.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Collection file loading configuration options.
    pub collections: Option<CollectionsConfig>,
    /// Schema file loading configuration options.
    pub schemas: Option<SchemasConfig>,
}

/// Server configuration settings such as port, static folder, and CORS.
///
/// These settings apply globally to the mock server.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    /// Port number the server listens on.
    pub port: Option<u16>,
    /// Filesystem path to serve static files from.
    pub folder: Option<String>,
    /// Enable or disable Cross-Origin Resource Sharing.
    pub enable_cors: Option<bool>,
    /// Allowed origin for CORS requests.
    pub allowed_origin: Option<String>,
    /// Enable HTTPS with a generated localhost certificate.
    pub ssl: Option<bool>,
    /// Path to a PEM-encoded TLS certificate.
    pub ssl_cert: Option<String>,
    /// Path to a PEM-encoded TLS private key.
    pub ssl_key: Option<String>,
}

/// Route-specific configuration settings.
///
/// Allows overriding default delay, remapping paths,
/// and protection for individual routes.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectionConfig {
    /// Name of the Fosk collection.
    pub name: Option<String>,
    /// Field name to use as the identifier key in the Fosk collection.
    pub id_key: Option<String>,
    /// Strategy for generating or interpreting Fosk collection identifiers.
    pub id_type: Option<IdType>,
}

/// Collection file loading configuration.
///
/// Defines where startup collection seed files are loaded from.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectionsConfig {
    /// Folder containing collection seed files, relative to the mock root unless absolute.
    pub folder: Option<String>,
}

/// Authentication-related configuration.
///
/// Includes user credentials, cookie settings, JWT secret,
/// and routes for login, logout, and user management.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Endpoint for user login.
    pub login_endpoint: Option<String>,
    /// Endpoint for user logout.
    pub logout_endpoint: Option<String>,
    /// Route path for user management.
    pub users_route: Option<String>,
}

/// File upload configuration settings.
///
/// Defines routes and behavior for uploading, downloading,
/// and listing files, including temporary storage options.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UploadConfig {
    /// Route path for handling file uploads.
    pub upload_endpoint: Option<String>,
    /// Route path for file downloads.
    pub download_endpoint: Option<String>,
    /// Route path for listing files.
    pub list_files_endpoint: Option<String>,
    /// Use temporary storage for uploads.
    pub temporary: Option<bool>,
}

/// Schema file loading configuration.
///
/// Defines where compact Fosk schema files are loaded from at startup.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemasConfig {
    /// Folder containing schema files, relative to the mock root unless absolute.
    pub folder: Option<String>,
    /// File name containing the complete database schema.
    pub db_schema: Option<String>,
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
        let content = fs::read_to_string(value.path()).map_err(|e| e.to_string())?;

        Config::try_from(content.as_str()).map_err(|e| e.to_string())
    }
}

/// Lookup table for TOML configuration files found in one directory.
#[derive(Debug, Default)]
pub struct ConfigStore {
    map_configs: HashMap<String, Config>,
}

impl ConfigStore {
    /// Loads all TOML files in a directory, keyed by lowercase file stem.
    pub fn try_from_dir(dir_path: &str) -> Result<Self, std::io::Error> {
        let mut store = Self::default();
        fs::read_dir(dir_path)?
            .filter_map(Result::ok)
            .filter(|file| is_toml(&file.file_name()))
            .for_each(|file| {
                let key = file
                    .path()
                    .as_path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_ascii_lowercase();
                match Config::try_from(&file) {
                    Ok(config) => {
                        store.map_configs.insert(key, config);
                    }
                    Err(err) => println!(
                        "Unable to load the config file {:?} due the error {}.",
                        file.file_name(),
                        err
                    ),
                }
            });

        Ok(store)
    }

    /// Returns a cloned configuration by case-insensitive key.
    pub fn get(&self, key: &str) -> Option<Config> {
        self.map_configs
            .get(key.to_ascii_lowercase().as_str())
            .cloned()
    }
}

/// Merge behavior where child values override parent defaults.
pub trait Mergeable {
    /// Merges this value with a parent value.
    fn merge(self, parent: Self) -> Self;
}

impl Config {
    /// Merges this configuration with an optional parent configuration.
    pub fn merge(self, parent: Option<Self>) -> Self {
        match parent {
            Some(parent) => Self {
                server: self.server.merge(parent.server),
                route: self.route.merge(parent.route),
                collection: self.collection, //.merge(parent.collection), don't merge collections
                auth: self.auth,             //.merge(parent.auth), don't merge auths
                upload: self.upload,         //.merge(parent.upload), don't merge upload
                collections: self.collections.merge(parent.collections),
                schemas: self.schemas.merge(parent.schemas),
            },
            None => self,
        }
    }

    /// Merges this configuration with a borrowed parent configuration.
    pub fn merge_with_ref(self, parent: &Self) -> Self {
        let parent = parent.clone();
        Self {
            server: self.server.merge(parent.server),
            route: self.route.merge(parent.route),
            collection: self.collection, //.merge(parent.collection), don't merge collections
            auth: self.auth,             //.merge(parent.auth), don't merge auths
            upload: self.upload,         //.merge(parent.upload), don't merge upload
            collections: self.collections.merge(parent.collections),
            schemas: self.schemas.merge(parent.schemas),
        }
    }

    /// Sets the route protection flag.
    pub fn with_protect(mut self, protect: bool) -> Self {
        let mut route = self.route.unwrap_or_default();
        route.protect = Some(protect);
        self.route = Some(route);

        self
    }

    /// Sets the collection name.
    pub fn with_collection_name(mut self, name: &str) -> Self {
        let mut collection = self.collection.unwrap_or_default();
        collection.name = Some(name.to_string());
        self.collection = Some(collection);

        self
    }

    /// Sets the collection identifier field.
    pub fn with_id_key(mut self, id_key: &str) -> Self {
        let mut collection = self.collection.unwrap_or_default();
        collection.id_key = Some(id_key.to_string());
        self.collection = Some(collection);

        self
    }

    /// Sets the collection identifier strategy.
    pub fn with_id_type(mut self, id_type: IdType) -> Self {
        let mut collection = self.collection.unwrap_or_default();
        collection.id_type = Some(id_type);
        self.collection = Some(collection);

        self
    }
}

impl Mergeable for Config {
    fn merge(self, parent: Self) -> Self {
        Self {
            server: self.server.merge(parent.server),
            route: self.route.merge(parent.route),
            collection: self.collection, //.merge(parent.collection), don't merge collections
            auth: self.auth,             //.merge(parent.auth), don't merge auths
            upload: self.upload,         //.merge(parent.upload), don't merge upload
            collections: self.collections.merge(parent.collections),
            schemas: self.schemas.merge(parent.schemas),
        }
    }
}

impl Mergeable for Option<Config> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (None, None) => None,
            (None, Some(p)) => Some(Config {
                route: None.merge(p.route),
                collections: None.merge(p.collections),
                schemas: None.merge(p.schemas),
                ..Default::default()
            }),
            (Some(child), None) => Some(child),
            (Some(child), Some(parent)) => Some(Config {
                server: child.server.merge(parent.server),
                route: child.route.merge(parent.route),
                collection: child.collection, //.merge(parent.collection), don't merge collections
                auth: child.auth,             //.merge(parent.auth), don't merge auths
                upload: child.upload,         //.merge(parent.upload), don't merge upload
                collections: child.collections.merge(parent.collections),
                schemas: child.schemas.merge(parent.schemas),
            }),
        }
    }
}

impl Mergeable for Option<ServerConfig> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (None, None) => None,
            (None, Some(p)) => Some(p),
            (Some(child), None) => Some(child),
            (Some(child), Some(parent)) => Some(ServerConfig {
                port: child.port.merge(parent.port),
                folder: child.folder.merge(parent.folder),
                enable_cors: child.enable_cors.merge(parent.enable_cors),
                allowed_origin: child.allowed_origin.merge(parent.allowed_origin),
                ssl: child.ssl.merge(parent.ssl),
                ssl_cert: child.ssl_cert.merge(parent.ssl_cert),
                ssl_key: child.ssl_key.merge(parent.ssl_key),
            }),
        }
    }
}

impl Mergeable for Option<RouteConfig> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (None, None) => None,
            (None, Some(p)) => Some(RouteConfig {
                delay: p.delay,
                protect: p.protect,
                ..Default::default()
            }),
            (Some(child), None) => Some(child),
            (Some(child), Some(parent)) => Some(RouteConfig {
                delay: child.delay.merge(parent.delay),
                remap: child.remap, //.merge(parent.remap),
                protect: child.protect.merge(parent.protect),
            }),
        }
    }
}

impl Mergeable for Option<CollectionConfig> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (None, None) => None,
            (None, Some(p)) => Some(p),
            (Some(child), None) => Some(child),
            (Some(child), Some(parent)) => Some(CollectionConfig {
                name: child.name.merge(parent.name),
                id_key: child.id_key.merge(parent.id_key),
                id_type: child.id_type.merge(parent.id_type),
            }),
        }
    }
}

impl Mergeable for Option<AuthConfig> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (None, None) => None,
            (None, Some(parent)) => Some(parent),
            (Some(child), None) => Some(child),
            (Some(child), Some(parent)) => Some(AuthConfig {
                username_field: child.username_field.merge(parent.username_field),
                password_field: child.password_field.merge(parent.password_field),
                roles_field: child.roles_field.merge(parent.roles_field),
                cookie_name: child.cookie_name.merge(parent.cookie_name),
                encrypt_password: child.encrypt_password.merge(parent.encrypt_password),
                jwt_secret: child.jwt_secret.merge(parent.jwt_secret),
                token_collection: child.token_collection.merge(parent.token_collection),
                user_collection: child.user_collection.merge(parent.user_collection),
                login_endpoint: child.login_endpoint.merge(parent.login_endpoint),
                logout_endpoint: child.logout_endpoint.merge(parent.logout_endpoint),
                users_route: child.users_route.merge(parent.users_route),
            }),
        }
    }
}

impl Mergeable for Option<UploadConfig> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (None, None) => None,
            (None, Some(parent)) => Some(parent),
            (Some(child), None) => Some(child),
            (Some(child), Some(parent)) => Some(UploadConfig {
                upload_endpoint: child.upload_endpoint.merge(parent.upload_endpoint),
                download_endpoint: child.download_endpoint.merge(parent.download_endpoint),
                list_files_endpoint: child.list_files_endpoint.merge(parent.list_files_endpoint),
                temporary: child.temporary.merge(parent.temporary),
            }),
        }
    }
}

impl Mergeable for Option<SchemasConfig> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (None, None) => None,
            (None, Some(p)) => Some(p),
            (Some(child), None) => Some(child),
            (Some(child), Some(parent)) => Some(SchemasConfig {
                folder: child.folder.merge(parent.folder),
                db_schema: child.db_schema.merge(parent.db_schema),
            }),
        }
    }
}

impl Mergeable for Option<CollectionsConfig> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (None, None) => None,
            (None, Some(p)) => Some(p),
            (Some(child), None) => Some(child),
            (Some(child), Some(parent)) => Some(CollectionsConfig {
                folder: child.folder.merge(parent.folder),
            }),
        }
    }
}

impl Mergeable for Option<String> {
    fn merge(self, parent: Self) -> Self {
        if self.is_some() { self } else { parent }
    }
}

impl Mergeable for Option<bool> {
    fn merge(self, parent: Self) -> Self {
        if self.is_some() { self } else { parent }
    }
}

impl Mergeable for Option<u16> {
    fn merge(self, parent: Self) -> Self {
        if self.is_some() { self } else { parent }
    }
}

impl Mergeable for Option<IdType> {
    fn merge(self, parent: Self) -> Self {
        if self.is_some() { self } else { parent }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_merge() {
        let child = ServerConfig {
            port: Some(3000),
            folder: None,
            enable_cors: Some(false),
            allowed_origin: None,
            ..Default::default()
        };
        let parent = ServerConfig {
            port: None,
            folder: Some("mocks".to_string()),
            enable_cors: Some(true),
            allowed_origin: Some("example.com".to_string()),
            ..Default::default()
        };
        let merged = Some(child.clone()).merge(Some(parent.clone())).unwrap();
        assert_eq!(merged.port, Some(3000));
        assert_eq!(merged.folder, Some("mocks".to_string()));
        assert_eq!(merged.enable_cors, Some(false));
        assert_eq!(merged.allowed_origin, Some("example.com".to_string()));
    }

    #[test]
    fn test_route_config_merge() {
        let child = RouteConfig {
            delay: None,
            remap: Some("/api".into()),
            protect: None,
        };
        let parent = RouteConfig {
            delay: Some(10),
            remap: None,
            protect: Some(true),
        };
        let merged = Some(child.clone()).merge(Some(parent.clone())).unwrap();
        assert_eq!(merged.delay, Some(10));
        assert_eq!(merged.remap, Some("/api".to_string()));
        assert_eq!(merged.protect, Some(true));
    }

    #[test]
    fn test_collection_config_merge() {
        let child = CollectionConfig {
            name: Some("child".into()),
            id_key: None,
            id_type: Some(IdType::Uuid),
        };
        let parent = CollectionConfig {
            name: None,
            id_key: Some("id".into()),
            id_type: Some(IdType::Int),
        };
        let merged = Some(child.clone()).merge(Some(parent.clone())).unwrap();
        assert_eq!(merged.name, Some("child".to_string()));
        assert_eq!(merged.id_key, Some("id".to_string()));
        assert_eq!(merged.id_type, Some(IdType::Uuid));
    }

    #[test]
    fn test_auth_config_merge() {
        let child = AuthConfig {
            username_field: Some("user".into()),
            token_collection: Some(CollectionConfig {
                name: Some("tok".into()),
                id_key: Some("t".into()),
                id_type: Some(IdType::Uuid),
            }),
            ..Default::default()
        };
        let parent = AuthConfig {
            username_field: Some("parent".into()),
            password_field: Some("pass".into()),
            token_collection: Some(CollectionConfig {
                name: Some("parent_tok".into()),
                id_key: None,
                id_type: Some(IdType::Int),
            }),
            ..Default::default()
        };
        let merged = Some(child.clone()).merge(Some(parent.clone())).unwrap();
        assert_eq!(merged.username_field, Some("user".into()));
        assert_eq!(merged.password_field, Some("pass".into()));
        let token = merged.token_collection.unwrap();
        assert_eq!(token.name, Some("tok".into()));
        assert_eq!(token.id_key, Some("t".into()));
        assert_eq!(token.id_type, Some(IdType::Uuid));
    }

    #[test]
    fn test_upload_config_merge() {
        let child = UploadConfig {
            upload_endpoint: None,
            download_endpoint: Some("/dl".into()),
            list_files_endpoint: None,
            temporary: Some(true),
        };
        let parent = UploadConfig {
            upload_endpoint: Some("/up".into()),
            download_endpoint: None,
            list_files_endpoint: Some("/list".into()),
            temporary: Some(false),
        };
        let merged = Some(child.clone()).merge(Some(parent.clone())).unwrap();
        assert_eq!(merged.upload_endpoint, Some("/up".into()));
        assert_eq!(merged.download_endpoint, Some("/dl".into()));
        assert_eq!(merged.list_files_endpoint, Some("/list".into()));
        assert_eq!(merged.temporary, Some(true));
    }

    #[test]
    fn test_config_option_merge() {
        let child = Config {
            server: Some(ServerConfig {
                port: Some(1),
                folder: None,
                enable_cors: None,
                allowed_origin: None,
                ..Default::default()
            }),
            route: None,
            collection: None,
            auth: None,
            upload: None,
            collections: None,
            schemas: None,
        };
        let parent = Config {
            server: Some(ServerConfig {
                port: None,
                folder: Some("dir".into()),
                enable_cors: Some(true),
                allowed_origin: Some("o".into()),
                ..Default::default()
            }),
            route: Some(RouteConfig {
                delay: Some(5),
                remap: None,
                protect: Some(false),
            }),
            collection: None,
            auth: None,
            upload: None,
            collections: None,
            schemas: None,
        };
        let merged_opt = Some(child.clone()).merge(Some(parent.clone()));
        let merged = merged_opt.unwrap();
        let server = merged.server.unwrap();
        assert_eq!(server.port, Some(1));
        assert_eq!(server.folder, Some("dir".into()));
        assert_eq!(server.enable_cors, Some(true));
        assert_eq!(
            merged.route,
            Some(RouteConfig {
                delay: Some(5),
                remap: None,
                protect: Some(false)
            })
        );
    }

    #[test]
    fn test_config_merge_trait() {
        let child = Config {
            server: None,
            route: Some(RouteConfig {
                delay: Some(2),
                remap: None,
                protect: None,
            }),
            collection: None,
            auth: None,
            upload: None,
            collections: None,
            schemas: None,
        };
        let parent = Config {
            server: None,
            route: Some(RouteConfig {
                delay: None,
                remap: Some("/p".into()),
                protect: Some(true),
            }),
            collection: None,
            auth: None,
            upload: None,
            collections: None,
            schemas: None,
        };
        let merged = child.merge(Some(parent));
        let route = merged.route.unwrap();
        assert_eq!(route.delay, Some(2));
        assert!(route.remap.is_none());
        assert_eq!(route.protect, Some(true));
    }

    #[test]
    fn test_collection_builder_helpers_and_owned_merge() {
        let config = Config::default()
            .with_collection_name("users")
            .with_id_key("_id")
            .with_id_type(IdType::Int);
        let collection = config.collection.as_ref().unwrap();
        assert_eq!(collection.name.as_deref(), Some("users"));
        assert_eq!(collection.id_key.as_deref(), Some("_id"));
        assert_eq!(collection.id_type, Some(IdType::Int));

        let parent = Config::default().with_protect(false);
        let child = Config::default().with_protect(true);
        let merged = <Config as Mergeable>::merge(child, parent);
        assert_eq!(merged.route.unwrap().protect, Some(true));
    }

    #[test]
    fn test_config_try_from_dir_entry() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_path = temp_dir.path().join("route.toml");
        std::fs::write(&config_path, "[route]\nprotect = true\n").unwrap();
        let entry = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| entry.path() == config_path)
            .unwrap();

        let config = Config::try_from(&entry).unwrap();
        assert_eq!(config.route.unwrap().protect, Some(true));
    }

    #[test]
    fn test_schemas_config_deserializes_and_merges() {
        let config = Config::try_from(
            r#"
            [schemas]
            folder = "schema-files"
            db_schema = "database.schema"
            "#,
        )
        .unwrap();
        assert_eq!(
            config.schemas,
            Some(SchemasConfig {
                folder: Some("schema-files".to_string()),
                db_schema: Some("database.schema".to_string()),
            })
        );

        let child = Some(SchemasConfig {
            folder: None,
            db_schema: Some("custom.schema".to_string()),
        });
        let parent = Some(SchemasConfig {
            folder: Some("{schemas}".to_string()),
            db_schema: Some("db.schema".to_string()),
        });
        let merged = child.merge(parent).unwrap();
        assert_eq!(merged.folder.as_deref(), Some("{schemas}"));
        assert_eq!(merged.db_schema.as_deref(), Some("custom.schema"));
    }

    #[test]
    fn test_collections_config_deserializes_and_merges() {
        let config = Config::try_from(
            r#"
            [collections]
            folder = "seed-data"
            "#,
        )
        .unwrap();
        assert_eq!(
            config.collections,
            Some(CollectionsConfig {
                folder: Some("seed-data".to_string()),
            })
        );

        let child = Some(CollectionsConfig { folder: None });
        let parent = Some(CollectionsConfig {
            folder: Some("{collections}".to_string()),
        });
        let merged = child.merge(parent).unwrap();
        assert_eq!(merged.folder.as_deref(), Some("{collections}"));
    }
}
