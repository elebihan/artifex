//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Certificate management.

use super::{
    pkcs11,
    uri::{self, Uri},
};
use thiserror::Error;
use tokio_rustls::rustls::pki_types::{CertificateDer, pem::PemObject};

/// Errors reported when hendling a X509 certificate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("PKCS#11 error: {0}")]
    Pkcs11(#[from] pkcs11::Error),
    #[error("URI error: {0}")]
    Uri(#[from] uri::Error),
    #[error("PEM error: {0}")]
    Pem(#[from] tokio_rustls::rustls::pki_types::pem::Error),
}

pub(crate) fn load_certificate<'a>(uri: &Uri) -> Result<CertificateDer<'a>, Error> {
    let cert = match uri {
        Uri::File(uri) => CertificateDer::from_pem_file(uri.path())?,
        Uri::Pkcs11(uri) => pkcs11::read_certificate(uri)
            .map_err(pkcs11::Error::Certificate)
            .map(CertificateDer::from)?,
    };
    Ok(cert)
}
