//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! URI management.

use thiserror::Error;

/// Errors reported when hendling an URI.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] url::ParseError),
    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),
}

pub use url::Url as Uri;
