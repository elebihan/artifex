//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use super::password::PasswordProvider;
use super::tls::Config as TlsConfig;
use artifex_batch::MarkupKind;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

/// Errors reported when managing the configuration.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Hold the configuration of the client.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    /// Markup kind to use as report format.
    pub format: MarkupKind,
    /// URL of the client.
    pub url: String,
    /// Password provider.
    #[serde(default = "PasswordProvider::default")]
    pub password_provider: PasswordProvider,
    /// Configuration of TLS.
    pub tls: TlsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: MarkupKind::Yaml,
            url: Self::DEFAULT_URL.to_string(),
            password_provider: PasswordProvider::Prompt,
            tls: TlsConfig {
                root_cert: Self::DEFAULT_TLS_ROOT_CERT.to_string(),
                client_cert: Self::DEFAULT_TLS_CLIENT_CERT.to_string(),
                client_key: Self::DEFAULT_TLS_CLIENT_KEY.to_string(),
                server_alt_name: None,
            },
        }
    }
}

impl Config {
    /// Default URL to connect to.
    pub const DEFAULT_URL: &str = "https://127.0.0.1:50051";
    /// Default TLS root certification authority file.
    pub const DEFAULT_TLS_ROOT_CERT: &str = "file:/etc/artifex/root.crt.pem";
    /// Default TLS client certificate.
    pub const DEFAULT_TLS_CLIENT_CERT: &str = "file:/etc/artifex/client.crt.pem";
    /// Default TLS client private key.
    pub const DEFAULT_TLS_CLIENT_KEY: &str = "file:/etc/artifex/client.key.pem";
    /// Create a new configuration from file at `path`.
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)?;
        let config = toml::de::from_str(&text)?;
        Ok(config)
    }
}
