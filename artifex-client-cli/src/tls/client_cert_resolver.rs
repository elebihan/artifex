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

use crate::tls::file_signing_key::FileSigningKeyBuilder;

// A builder for configuring a client client certificate resolver.
#[derive(Debug)]
pub(super) struct ClientCertResolverBuilder {
    cert_pem: String,
    key_builder: FileSigningKeyBuilder,
}

impl ClientCertResolverBuilder {
    /// Create a builder.
    pub(super) fn with_pem_files<P: AsRef<Path>>(cert_path: P, key_path: P) -> Result<Self> {
        let cert_pem = std::fs::read_to_string(&cert_path).with_context(|| {
            format!(
                "Failed to read certificate from '{}'",
                cert_path.as_ref().display()
            )
        })?;
        let key_builder = FileSigningKeyBuilder::with_pem_file(&key_path).with_context(|| {
            format!("Failed to read key from '{}'", key_path.as_ref().display())
        })?;
        Ok(Self {
            cert_pem,
            key_builder,
        })
    }
    /// Set password for key decryption.
    pub(super) fn password(&mut self, password: &str) -> &Self {
        self.key_builder.password(password);
        self
    }
    /// Build a client certificate resolver.
    pub(super) fn build(self) -> Result<ClientCertResolver> {
        let cert = CertificateDer::from_pem_reader(self.cert_pem.as_bytes())
            .with_context(|| "Failed to create certificate")?;
        let key = self
            .key_builder
            .build()
            .with_context(|| "Failed to create client signing key")?;
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
