//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use tonic::transport::{Certificate, Identity, ServerTlsConfig};

/// Hold the configuration of the TLS.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    /// Path to root certification authority file.
    pub root_cert: PathBuf,
    /// Path to server certificate file.
    pub server_cert: PathBuf,
    /// Path to server private key file.
    pub server_key: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_cert: PathBuf::from(Self::DEFAULT_ROOT_CERT),
            server_cert: PathBuf::from(Self::DEFAULT_SERVER_CERT),
            server_key: PathBuf::from(Self::DEFAULT_SERVER_KEY),
        }
    }
}

impl Config {
    /// Default root certification authoritity file.
    pub const DEFAULT_ROOT_CERT: &str = "/etc/artifex/root.pem";
    /// Default server certificate.
    pub const DEFAULT_SERVER_CERT: &str = "/etc/artifex/server.crt.pem";
    /// Default server private key.
    pub const DEFAULT_SERVER_KEY: &str = "/etc/artifex/server.key.pem";
}

/// Create TLS server configuration.
pub fn create_server_config(config: &Config) -> Result<ServerTlsConfig> {
    let root_cert = std::fs::read_to_string(&config.root_cert).with_context(|| {
        format!(
            "Failed to read root certificate from '{}'",
            config.root_cert.display()
        )
    })?;
    let root_cert = Certificate::from_pem(root_cert);
    let cert = std::fs::read_to_string(&config.server_cert).with_context(|| {
        format!(
            "Failed to read server certificate from '{}'",
            config.server_cert.display()
        )
    })?;
    let key = std::fs::read_to_string(&config.server_key).with_context(|| {
        format!(
            "Failed to read server private key from '{}'",
            config.server_key.display()
        )
    })?;
    let identity = Identity::from_pem(cert, key);
    let config = ServerTlsConfig::new()
        .client_ca_root(root_cert)
        .identity(identity);
    Ok(config)
}
