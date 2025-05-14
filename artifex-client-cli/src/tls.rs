//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use tonic::transport::{Certificate, ClientTlsConfig, Identity};

/// Hold the configuration of the TLS.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    /// Path to root certification authority file.
    pub root_cert: PathBuf,
    /// Path to client certificate file.
    pub client_cert: PathBuf,
    /// Path to client private key file.
    pub client_key: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_cert: PathBuf::from(Self::DEFAULT_ROOT_CERT),
            client_cert: PathBuf::from(Self::DEFAULT_CLIENT_CERT),
            client_key: PathBuf::from(Self::DEFAULT_CLIENT_KEY),
        }
    }
}

impl Config {
    /// Default root certification authoritity file.
    pub const DEFAULT_ROOT_CERT: &str = "/etc/artifex/root.crt.pem";
    /// Default client certificate.
    pub const DEFAULT_CLIENT_CERT: &str = "/etc/artifex/client.crt.pem";
    /// Default client private key.
    pub const DEFAULT_CLIENT_KEY: &str = "/etc/artifex/client.key.pem";
}

/// Create TLS client configuration.
pub fn create_client_config(config: &Config) -> Result<ClientTlsConfig> {
    let root_cert = std::fs::read_to_string(&config.root_cert).with_context(|| {
        format!(
            "Failed to read root certificate from '{}'",
            config.root_cert.display()
        )
    })?;
    let root_cert = Certificate::from_pem(root_cert);
    let cert = std::fs::read_to_string(&config.client_cert).with_context(|| {
        format!(
            "Failed to read client certificate from '{}'",
            config.client_cert.display()
        )
    })?;
    let key = std::fs::read_to_string(&config.client_key).with_context(|| {
        format!(
            "Failed to read client private key from '{}'",
            config.client_key.display()
        )
    })?;
    let identity = Identity::from_pem(cert, key);
    let config = ClientTlsConfig::new()
        //
        // The certificate of the server should contain the following in
        // 'X509v3 extensions' section:
        //     X509v3 Subject Alternative Name:
        //     DNS:artifex-server
        //
        .domain_name("artifex-server")
        .ca_certificate(root_cert)
        .identity(identity);
    Ok(config)
}
