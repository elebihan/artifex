//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::sync::Arc;
use thiserror::Error;
use tokio_rustls::rustls::{client::ResolvesClientCert, sign::CertifiedKey, SignatureScheme};

use crate::tls::file_signing_key::FileSigningKeyBuilder;

use super::cert;
use super::uri::{self, Uri};

/// Errors occuring when operating with a client certificate resolver.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Certificate error: {0}")]
    Certificate(#[from] cert::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Signing key error: {0}")]
    SigningKey(#[from] crate::tls::file_signing_key::Error),
    #[error("URI error: {0}")]
    Uri(#[from] crate::tls::uri::Error),
}

// A builder for configuring a client client certificate resolver.
#[derive(Debug)]
pub(super) struct ClientCertResolverBuilder {
    cert_uri: String,
    key_builder: FileSigningKeyBuilder,
}

impl ClientCertResolverBuilder {
    /// Create a builder.
    pub(super) fn new(cert_uri: &str, key_uri: &str) -> Result<Self, Error> {
        let key_uri = Uri::parse(key_uri).map_err(uri::Error::InvalidUri)?;
        let key_builder = match key_uri.scheme() {
            "file" | "data" => FileSigningKeyBuilder::with_pem_file(&key_uri.path())?,
            s => return Err(Error::Uri(uri::Error::UnsupportedScheme(s.to_string()))),
        };
        Ok(Self {
            cert_uri: cert_uri.to_string(),
            key_builder,
        })
    }
    /// Set password for key decryption.
    pub(super) fn password(&mut self, password: &str) -> &Self {
        self.key_builder.password(password);
        self
    }
    /// Build a client certificate resolver.
    pub(super) fn build(self) -> Result<ClientCertResolver, Error> {
        let cert = cert::load_certificate(&self.cert_uri)?;
        let key = self.key_builder.build()?;
        let key = CertifiedKey::new(vec![cert], Arc::new(key));
        Ok(ClientCertResolver { key: Arc::new(key) })
    }
}

/// Choose the certificate chain and private key for client authentication.
#[derive(Debug)]
pub(super) struct ClientCertResolver {
    key: Arc<CertifiedKey>,
}

impl ResolvesClientCert for ClientCertResolver {
    fn has_certs(&self) -> bool {
        true
    }

    fn resolve(
        &self,
        _root_hint_subjects: &[&[u8]],
        _sigschemes: &[SignatureScheme],
    ) -> Option<Arc<CertifiedKey>> {
        Some(self.key.clone())
    }
}
