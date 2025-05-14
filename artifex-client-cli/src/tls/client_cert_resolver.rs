//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tokio_rustls::rustls::{
    client::ResolvesClientCert,
    pki_types::{pem::PemObject, CertificateDer},
    sign::CertifiedKey,
    SignatureScheme,
};

use super::client_signing_key::ClientSigningKey;

/// Choose the certificate chain and private key for client authentication.
#[derive(Debug)]
pub(super) struct ClientCertResolver {
    key: Arc<CertifiedKey>,
}

impl ClientCertResolver {
    /// Create a new client certificate resolver.
    pub(super) fn with_pem_files<P: AsRef<Path>>(cert_path: P, key_path: P) -> Result<Self> {
        let cert = CertificateDer::from_pem_file(&cert_path).with_context(|| {
            format!(
                "Failed to create certificate from '{}'",
                cert_path.as_ref().display()
            )
        })?;
        let key = ClientSigningKey::with_pem_file(&key_path).with_context(|| {
            format!(
                "Failed to create pub key from {}",
                key_path.as_ref().display()
            )
        })?;
        let key = CertifiedKey::new(vec![cert], Arc::new(key));
        Ok(Self { key: Arc::new(key) })
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
