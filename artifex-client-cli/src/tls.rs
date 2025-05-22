//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! TLS client configuration.

mod client_cert_resolver;
mod file_signing_key;

use anyhow::{Context, Result};
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc};
use tokio_rustls::rustls::{
    self,
    pki_types::{pem::PemObject, CertificateDer},
    ClientConfig, RootCertStore,
};

use client_cert_resolver::ClientCertResolverBuilder;

/// Hold the configuration of the TLS.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    /// Path to root certification authority file.
    pub root_cert: PathBuf,
    /// Path to client certificate file.
    pub client_cert: PathBuf,
    /// Path to client private key file.
    pub client_key: PathBuf,
    /// Password for client private key file.
    pub client_password: Option<String>,
    /// Server alternative name.
    pub server_alt_name: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_cert: PathBuf::from(Self::DEFAULT_ROOT_CERT),
            client_cert: PathBuf::from(Self::DEFAULT_CLIENT_CERT),
            client_key: PathBuf::from(Self::DEFAULT_CLIENT_KEY),
            client_password: None,
            server_alt_name: None,
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
pub fn create_client_config(config: &Config) -> Result<ClientConfig> {
    let mut ca_store = RootCertStore::empty();
    let root_cert = CertificateDer::from_pem_file(&config.root_cert).with_context(|| {
        format!(
            "Failed to read root certificate from '{}'",
            config.root_cert.display()
        )
    })?;
    ca_store
        .add(root_cert)
        .with_context(|| "Failed to add root certificate")?;
    let provider = rustls::crypto::ring::default_provider();
    let mut client_cert_resolver_builder =
        ClientCertResolverBuilder::with_pem_files(&config.client_cert, &config.client_key)?;
    if let Some(password) = &config.client_password {
        client_cert_resolver_builder.password(password);
    }
    let client_cert_resolver = client_cert_resolver_builder
        .build()
        .with_context(|| "Failed to create client certificate resolver")?;
    let tls = ClientConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .with_context(|| "Failed to get protocols")?;
    let tls = tls
        .with_root_certificates(ca_store)
        .with_client_cert_resolver(Arc::new(client_cert_resolver));
    Ok(tls)
}
