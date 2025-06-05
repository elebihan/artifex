//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! TLS client configuration.

mod cert;
mod client_cert_resolver;
mod file;
mod uri;

use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;
use tokio_rustls::rustls::{self, pki_types, ClientConfig, RootCertStore};

use client_cert_resolver::ClientCertResolver;
use uri::Uri;

/// Errors occuring when configuring TLS connection.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Certificate error: {0}")]
    Certificate(#[from] cert::Error),
    #[error("Client certificate resolver error: {0}")]
    ClientCertResolver(#[from] client_cert_resolver::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PEM error: {0}")]
    Pem(#[from] pki_types::pem::Error),
    #[error("RusTLS error: {0}")]
    RusTls(#[from] rustls::Error),
    #[error("URI error: {0}")]
    Uri(#[from] uri::Error),
}

/// Hold the configuration of the TLS.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    /// URI for root certification authority.
    pub root_cert: String,
    /// URI for client certificate.
    pub client_cert: String,
    /// URI for client private key.
    pub client_key: String,
    /// Server alternative name.
    pub server_alt_name: Option<String>,
}

/// Create TLS client configuration.
pub fn create_client_config(config: &Config) -> Result<ClientConfig, Error> {
    let mut ca_store = RootCertStore::empty();
    let root_cert_uri = config.root_cert.parse::<Uri>()?;
    let root_cert = cert::load_certificate(&root_cert_uri)?;
    ca_store.add(root_cert)?;
    let client_cert_uri = config.client_cert.parse::<Uri>()?;
    let client_key_uri = config.client_key.parse::<Uri>()?;
    let client_key_uri = client_key_uri.source_secret()?;
    let client_cert_resolver = ClientCertResolver::new(&client_cert_uri, &client_key_uri)?;
    let provider = rustls::crypto::ring::default_provider();
    let tls = ClientConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()?;
    let tls = tls
        .with_root_certificates(ca_store)
        .with_client_cert_resolver(Arc::new(client_cert_resolver));
    Ok(tls)
}
