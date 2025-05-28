//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Certificate management.

use super::uri::{self, Uri};
use thiserror::Error;
use tokio_rustls::rustls::pki_types::{pem::PemObject, CertificateDer};

/// Errors reported when hendling a X509 certificate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("URI error: {0}")]
    Uri(#[from] uri::Error),
    #[error("PEM error: {0}")]
    Pem(#[from] tokio_rustls::rustls::pki_types::pem::Error),
}

pub(crate) fn load_certificate<'a>(uri: &str) -> Result<CertificateDer<'a>, Error> {
    let uri = Uri::parse(uri).map_err(uri::Error::InvalidUri)?;
    let cert = match uri.scheme() {
        "file" | "data" => CertificateDer::from_pem_file(&uri.path())?,
        s => return Err(Error::Uri(uri::Error::UnsupportedScheme(s.to_string()))),
    };
    Ok(cert)
}
