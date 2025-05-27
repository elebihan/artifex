//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! TLS client configuration.

mod client_cert_resolver;
mod file_signing_key;

use serde::Deserialize;
use std::{path::PathBuf, sync::Arc};
use thiserror::Error;
use tokio_rustls::rustls::{
    self,
    pki_types::{self, pem::PemObject, CertificateDer},
    ClientConfig, RootCertStore,
};

use client_cert_resolver::ClientCertResolverBuilder;

/// Errors occuring when configuring TLS connection.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Client certificate resolver error: {0}")]
    ClientCertResolver(#[from] client_cert_resolver::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PEM error: {0}")]
    Pem(#[from] pki_types::pem::Error),
    #[error("RusTLS error: {0}")]
    RusTls(#[from] rustls::Error),
}

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

/// Create TLS client configuration.
pub fn create_client_config(config: &Config) -> Result<ClientConfig, Error> {
    let mut ca_store = RootCertStore::empty();
    let root_cert = CertificateDer::from_pem_file(&config.root_cert)?;
    ca_store.add(root_cert)?;
    let provider = rustls::crypto::ring::default_provider();
    let mut client_cert_resolver_builder =
        ClientCertResolverBuilder::with_pem_files(&config.client_cert, &config.client_key)?;
    if let Some(password) = &config.client_password {
        client_cert_resolver_builder.password(password);
    }
    let client_cert_resolver = client_cert_resolver_builder.build()?;
    let tls = ClientConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()?;
    let tls = tls
        .with_root_certificates(ca_store)
        .with_client_cert_resolver(Arc::new(client_cert_resolver));
    Ok(tls)
}
