//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Manage certificates and keys on PKCS#11 cryptographic tokens.

mod cert;
pub(crate) mod signing_key;
mod uri;

use thiserror::Error;

/// Errors reported when handling PKCS#11 items.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Certificate error: {0}")]
    Certificate(#[from] cert::Error),
}

pub use cert::read_certificate;
pub use uri::Pkcs11Uri;
