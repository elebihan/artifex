//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Refer to cryptographic items on PKCS#11 cryptographic tokens via URI.

use crate::tls::uri::Error;
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

#[derive(Debug)]
struct Pkcs11Params(HashMap<String, String>);

impl FromStr for Pkcs11Params {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let items = s.split(';');
        let result: std::result::Result<HashMap<String, String>, Self::Err> = items
            .map(|kv| {
                kv.find('=')
                    .ok_or(Error::InvalidUri("Malformed parameter".to_string()))
                    .map(move |p| (kv[0..p].to_string(), kv[p + 1..].replace("%20", " ")))
            })
            .collect();
        Ok(Pkcs11Params(result?))
    }
}

impl TryFrom<&Url> for Pkcs11Params {
    type Error = Error;

    fn try_from(url: &Url) -> std::result::Result<Self, Self::Error> {
        let params = url.path().parse::<Pkcs11Params>()?;
        Ok(params)
    }
}

/// Represent the URI identifying a PKCS#11 object stored on a PKCS#11 token.
#[derive(Debug, Clone, PartialEq)]
pub struct Pkcs11Uri {
    pub(crate) module_path: String,
    pub(crate) token: String,
    pub(crate) object: String,
    pub(crate) pin: Option<String>,
    pub(crate) pin_source: Option<String>,
}

impl Pkcs11Uri {
    /// Return the absolute path to the PKCS#11 module to use.
    pub fn module_path(&self) -> &str {
        &self.module_path
    }
    /// Return the name of the PKCS#11 token to use.
    #[allow(dead_code)]
    pub fn token(&self) -> &str {
        &self.token
    }
    /// Return the name of the PKCS#11 object to use.
    pub fn object(&self) -> &str {
        &self.object
    }
    /// Return the PIN of the PKCS#11 object to use.
    pub fn pin(&self) -> Option<&str> {
        self.pin.as_deref()
    }
    /// Return the PIN source of the PKCS#11 object to use.
    pub fn pin_source(&self) -> Option<&str> {
        self.pin_source.as_deref()
    }
}

impl TryFrom<&Url> for Pkcs11Uri {
    type Error = Error;

    fn try_from(url: &Url) -> std::result::Result<Self, Self::Error> {
        let mut params = Pkcs11Params::try_from(url)?;
        let mut module_path = None;
        let mut pin = None;
        let mut pin_source = None;
        let pairs = url.query_pairs();
        for (k, v) in pairs {
            match k.as_ref() {
                "module-path" => module_path = Some(v.into()),
                "pin-value" => pin = Some(v.into()),
                "pin-source" => pin_source = Some(v.into()),
                _ => {}
            }
        }
        Ok(Pkcs11Uri {
            module_path: module_path
                .ok_or_else(|| Error::InvalidUri("Missing module-path".to_string()))?,
            token: params
                .0
                .remove("token")
                .ok_or_else(|| Error::InvalidUri("Missing token".to_string()))?,
            object: params
                .0
                .remove("object")
                .ok_or_else(|| Error::InvalidUri("Missing object".to_string()))?,
            pin,
            pin_source,
        })
    }
}

impl FromStr for Pkcs11Uri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uri = Url::parse(s)?;
        let uri = Pkcs11Uri::try_from(&uri)?;
        Ok(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_URI: &str = "pkcs11:token=Artifex%20Client%20Token%2002;object=Artifex%20Client%20Key%2002?module-path=/usr/lib64/libsofthsm2.so&pin-source=env:CLIENT_KEY_PASSWORD";

    #[test]
    fn try_from_valid() {
        let url = Url::parse(VALID_URI).unwrap();
        let res = Pkcs11Uri::try_from(&url);
        let reference = Pkcs11Uri {
            module_path: "/usr/lib64/libsofthsm2.so".to_string(),
            token: "Artifex Client Token 02".to_string(),
            object: "Artifex Client Key 02".to_string(),
            pin: None,
            pin_source: Some("env:CLIENT_KEY_PASSWORD".to_string()),
        };
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), reference);
    }
}
