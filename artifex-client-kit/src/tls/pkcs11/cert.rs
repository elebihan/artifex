//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Manage certificates on PKCS#11 cryptographic tokens.

use crate::tls::pkcs11::Pkcs11Uri;
use cryptoki::{
    context::{CInitializeArgs, Pkcs11},
    object::{Attribute, AttributeType, CertificateType},
};
use thiserror::Error;

/// Errors reported when handling certificates on PKCS#11 tokens.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Cryptoki error: {0}")]
    Cryptoki(#[from] cryptoki::error::Error),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Return the certificate matching a given URI.
pub fn read_certificate(uri: &Pkcs11Uri) -> Result<Vec<u8>, Error> {
    let pkcs11 = Pkcs11::new(uri.module_path())?;
    pkcs11.initialize(CInitializeArgs::OsThreads)?;
    let slots = pkcs11.get_slots_with_initialized_token()?;
    if slots.is_empty() {
        return Err(Error::NotFound("No PKCS#11 token found".to_string()));
    }
    let session = pkcs11.open_ro_session(slots[0])?;
    let cert_template = [
        Attribute::Label(uri.object().as_bytes().to_vec()),
        Attribute::CertificateType(CertificateType::X_509),
        Attribute::Token(true),
    ];
    let certs = session.find_objects(&cert_template)?;
    if certs.is_empty() {
        return Err(Error::NotFound("No such PKCS#11 certificate".to_string()));
    }
    let attrs = session.get_attributes(certs[0], &[AttributeType::Value])?;
    if attrs.is_empty() {
        return Err(Error::InvalidData(
            "PKCS#11 object has no value".to_string(),
        ));
    }
    if let Attribute::Value(value) = &attrs[0] {
        Ok(value.to_owned())
    } else {
        Err(Error::InvalidData(
            "PKCS#11 object attribute is not a value".to_string(),
        ))
    }
}
