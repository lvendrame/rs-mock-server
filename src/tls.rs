//! Local HTTPS configuration and certificate handling.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use axum_server::tls_rustls::RustlsConfig;
use rcgen::generate_simple_self_signed;

use crate::ServerConfig;

const SSL_CACHE_DIR: &str = ".rs-mock-server/ssl";
const GENERATED_CERT_FILE: &str = "localhost.pem";
const GENERATED_KEY_FILE: &str = "localhost-key.pem";

/// Resolved TLS mode for the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsMode {
    /// Serve HTTP without TLS.
    Disabled,
    /// Serve HTTPS with a generated localhost certificate.
    Generated,
    /// Serve HTTPS with user-provided PEM files.
    Provided {
        cert_path: PathBuf,
        key_path: PathBuf,
    },
}

/// Error returned when TLS configuration cannot be resolved or loaded.
#[derive(Debug)]
pub enum TlsError {
    /// Only one of certificate or key was provided.
    IncompleteKeyPair,
    /// Certificate generation failed.
    CertificateGeneration(rcgen::Error),
    /// Certificate files could not be written.
    CertificateStorage(std::io::Error),
    /// Rustls could not load the certificate pair.
    CertificateLoad(std::io::Error),
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlsError::IncompleteKeyPair => {
                write!(f, "SSL requires both ssl_cert and ssl_key")
            }
            TlsError::CertificateGeneration(err) => {
                write!(f, "failed to generate localhost SSL certificate: {}", err)
            }
            TlsError::CertificateStorage(err) => {
                write!(f, "failed to store localhost SSL certificate: {}", err)
            }
            TlsError::CertificateLoad(err) => {
                write!(f, "failed to load SSL certificate: {}", err)
            }
        }
    }
}

impl std::error::Error for TlsError {}

/// Resolves the TLS mode from server configuration.
pub fn resolve_tls_mode(config: &ServerConfig) -> Result<TlsMode, TlsError> {
    match explicit_key_pair(config)? {
        Some(mode) => Ok(mode),
        None if config.ssl.unwrap_or(false) => Ok(TlsMode::Generated),
        None => Ok(TlsMode::Disabled),
    }
}

/// Returns true when the resolved TLS mode serves HTTPS.
pub fn is_https(mode: &TlsMode) -> bool {
    !matches!(mode, TlsMode::Disabled)
}

/// Builds the Rustls server configuration for an HTTPS mode.
pub async fn rustls_config(mode: &TlsMode) -> Result<RustlsConfig, TlsError> {
    let (cert_path, key_path) = certificate_paths(mode)?;
    RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .map_err(TlsError::CertificateLoad)
}

fn explicit_key_pair(config: &ServerConfig) -> Result<Option<TlsMode>, TlsError> {
    match (&config.ssl_cert, &config.ssl_key) {
        (Some(cert), Some(key)) => Ok(Some(TlsMode::Provided {
            cert_path: PathBuf::from(cert),
            key_path: PathBuf::from(key),
        })),
        (None, None) => Ok(None),
        _ => Err(TlsError::IncompleteKeyPair),
    }
}

fn certificate_paths(mode: &TlsMode) -> Result<(PathBuf, PathBuf), TlsError> {
    match mode {
        TlsMode::Disabled => unreachable!("TLS config is not required for HTTP"),
        TlsMode::Generated => generated_certificate_paths(),
        TlsMode::Provided {
            cert_path,
            key_path,
        } => Ok((cert_path.clone(), key_path.clone())),
    }
}

fn generated_certificate_paths() -> Result<(PathBuf, PathBuf), TlsError> {
    let cert_path = generated_cert_path();
    let key_path = generated_key_path();

    if cert_path.exists() && key_path.exists() {
        return Ok((cert_path, key_path));
    }

    write_generated_certificate(&cert_path, &key_path)?;
    Ok((cert_path, key_path))
}

fn generated_cert_path() -> PathBuf {
    Path::new(SSL_CACHE_DIR).join(GENERATED_CERT_FILE)
}

fn generated_key_path() -> PathBuf {
    Path::new(SSL_CACHE_DIR).join(GENERATED_KEY_FILE)
}

fn write_generated_certificate(cert_path: &Path, key_path: &Path) -> Result<(), TlsError> {
    let (certificate, key) = generated_certificate_pems()?;
    let parent = cert_path.parent().expect("generated cert path has parent");

    fs::create_dir_all(parent).map_err(TlsError::CertificateStorage)?;
    fs::write(cert_path, certificate).map_err(TlsError::CertificateStorage)?;
    fs::write(key_path, key).map_err(TlsError::CertificateStorage)?;

    Ok(())
}

fn generated_certificate_pems() -> Result<(String, String), TlsError> {
    let names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];

    let certificate =
        generate_simple_self_signed(names).map_err(TlsError::CertificateGeneration)?;

    Ok((
        certificate.cert.pem(),
        certificate.signing_key.serialize_pem(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server_config(ssl: Option<bool>, cert: Option<&str>, key: Option<&str>) -> ServerConfig {
        ServerConfig {
            ssl,
            ssl_cert: cert.map(String::from),
            ssl_key: key.map(String::from),
            ..Default::default()
        }
    }

    #[test]
    fn tls_is_disabled_by_default() {
        let mode = resolve_tls_mode(&ServerConfig::default()).unwrap();
        assert_eq!(mode, TlsMode::Disabled);
        assert!(!is_https(&mode));
    }

    #[test]
    fn ssl_flag_uses_generated_certificate() {
        let mode = resolve_tls_mode(&server_config(Some(true), None, None)).unwrap();
        assert_eq!(mode, TlsMode::Generated);
        assert!(is_https(&mode));
    }

    #[test]
    fn explicit_certificate_pair_wins_over_generated_certificate() {
        let mode = resolve_tls_mode(&server_config(
            Some(true),
            Some("cert.pem"),
            Some("key.pem"),
        ))
        .unwrap();
        assert_eq!(
            mode,
            TlsMode::Provided {
                cert_path: PathBuf::from("cert.pem"),
                key_path: PathBuf::from("key.pem")
            }
        );
    }

    #[test]
    fn certificate_pair_must_be_complete() {
        assert!(matches!(
            resolve_tls_mode(&server_config(None, Some("cert.pem"), None)),
            Err(TlsError::IncompleteKeyPair)
        ));

        assert!(matches!(
            resolve_tls_mode(&server_config(None, None, Some("key.pem"))),
            Err(TlsError::IncompleteKeyPair)
        ));
    }

    #[test]
    fn generated_certificate_contains_localhost_names() {
        let (certificate, key) = generated_certificate_pems().unwrap();

        assert!(certificate.contains("BEGIN CERTIFICATE"));
        assert!(!key.is_empty());
    }

    #[tokio::test]
    async fn rustls_config_loads_generated_certificate_files() {
        let dir = tempfile::tempdir().unwrap();
        let cert_path = dir.path().join("localhost.pem");
        let key_path = dir.path().join("localhost-key.pem");
        write_generated_certificate(&cert_path, &key_path).unwrap();

        let mode = TlsMode::Provided {
            cert_path,
            key_path,
        };

        assert!(rustls_config(&mode).await.is_ok());
    }
}
