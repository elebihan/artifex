//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Refer to cryptograhic items in files via URI.

use crate::tls::uri::Error;
use std::path::{Path, PathBuf};
use url::Url;

/// Represent the URI identifying a cryptographic object stored in a file.

#[derive(Debug, Clone, PartialEq)]
pub struct FileUri {
    pub(crate) path: PathBuf,
    pub(crate) password: Option<String>,
    pub(crate) password_source: Option<String>,
}

impl FileUri {
    /// Return the path of the cryptographic object.
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }
    /// Return the password for accessing the cryptographic object.
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }
    /// Return the source of the password for accessing the cryptographic object.
    pub fn password_source(&self) -> Option<&str> {
        self.password_source.as_deref()
    }
}

impl TryFrom<&Url> for FileUri {
    type Error = Error;

    fn try_from(url: &Url) -> std::result::Result<Self, Self::Error> {
        if url.scheme() != "file" && url.scheme() != "data" {
            return Err(Error::InvalidUri("Invalid scheme".to_string()));
        }
        let path = PathBuf::from(url.path());
        let mut password = None;
        let mut password_source = None;
        let pairs = url.query_pairs();
        for (k, v) in pairs {
            match k.as_ref() {
                "password" => password = Some(v.into()),
                "password-source" => password_source = Some(v.into()),
                _ => {}
            }
        }
        Ok(Self {
            path,
            password,
            password_source,
        })
    }
}

impl std::str::FromStr for FileUri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uri = Url::parse(s)?;
        let uri = FileUri::try_from(&uri)?;
        Ok(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_URI: &str = "file:/some/where/client.key.pem?password-source=env:SOME_SECRET";

    #[test]
    fn try_from_valid() {
        let url = Url::parse(VALID_URI).unwrap();
        let res = FileUri::try_from(&url);
        let reference = FileUri {
            path: PathBuf::from("/some/where/client.key.pem"),
            password: None,
            password_source: Some("env:SOME_SECRET".to_string()),
        };
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), reference);
    }
}
