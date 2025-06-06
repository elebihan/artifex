//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::sync::Arc;
use thiserror::Error;
use tokio_rustls::rustls::{client::ResolvesClientCert, sign::CertifiedKey, SignatureScheme};

use super::cert;
use super::file::signing_key::FileSigningKey;
use super::uri::Uri;

/// Errors occuring when operating with a client certificate resolver.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Certificate error: {0}")]
    Certificate(#[from] cert::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Signing key error: {0}")]
    SigningKey(#[from] crate::tls::file::signing_key::Error),
    #[error("URI error: {0}")]
    Uri(#[from] crate::tls::uri::Error),
}

/// Choose the certificate chain and private key for client authentication.
#[derive(Debug)]
pub(super) struct ClientCertResolver {
    key: Arc<CertifiedKey>,
}

impl ClientCertResolver {
    /// Create a new client certificate resolver.
    pub(super) fn new(cert_uri: &Uri, key_uri: &Uri) -> Result<Self, Error> {
        let cert = cert::load_certificate(&cert_uri)?;
        let Uri::File(key_uri) = key_uri;
        let key = FileSigningKey::new(&key_uri)?;
        let key = CertifiedKey::new(vec![cert], Arc::new(key));
        Ok(ClientCertResolver { key: Arc::new(key) })
    }
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
