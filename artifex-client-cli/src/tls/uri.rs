//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! URI management.

use crate::password::PasswordProvider;

use super::file::FileUri;
use super::pkcs11::Pkcs11Uri;
use thiserror::Error;
use url::Url;

/// Errors reported when handling an URI.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    #[error("Password provider error: {0}")]
    PasswordProvider(#[from] crate::password::Error),
    #[error("PKCS#11 error {0}")]
    Pkcs11(#[from] super::pkcs11::Error),
    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
}

/// Represent the URI identifying a cryptographic object.
#[derive(Debug, Clone, PartialEq)]
pub enum Uri {
    File(FileUri),
    Pkcs11(Pkcs11Uri),
}

impl std::str::FromStr for Uri {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uri = Url::parse(s)?;
        let uri = match uri.scheme() {
            "file" | "data" => {
                let uri = FileUri::try_from(&uri)?;
                Uri::File(uri)
            }
            "pkcs11" => {
                let uri = Pkcs11Uri::try_from(&uri)?;
                Uri::Pkcs11(uri)
            }
            s => return Err(Error::UnsupportedScheme(s.to_string())),
        };
        Ok(uri)
    }
}

impl Uri {
    /// Check if URI contains a secret-sourcing member
    /// and if so return a URI where the secret member
    /// has been sourced.
    /// If the URI already contains a secret, it is left untouched.
    pub fn source_secret(self) -> Result<Uri, Error> {
        match self {
            Uri::File(uri) => {
                if uri.password().is_none() {
                    let password = if let Some(source) = uri.password_source() {
                        let provider = source.parse::<PasswordProvider>()?;
                        let password = provider.provide()?;
                        Some(password)
                    } else {
                        None
                    };
                    Ok(Uri::File(FileUri { password, ..uri }))
                } else {
                    Ok(Uri::File(uri))
                }
            }
            Uri::Pkcs11(uri) => {
                if uri.pin().is_none() {
                    let pin = if let Some(source) = uri.pin_source() {
                        let provider = source.parse::<PasswordProvider>()?;
                        let pin = provider.provide()?;
                        Some(pin)
                    } else {
                        None
                    };
                    Ok(Uri::Pkcs11(Pkcs11Uri { pin, ..uri }))
                } else {
                    Ok(Uri::Pkcs11(uri))
                }
            }
        }
    }
}
