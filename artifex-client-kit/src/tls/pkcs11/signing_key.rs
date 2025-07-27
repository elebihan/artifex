//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use super::Pkcs11Uri;
use cryptoki::{
    context::{CInitializeArgs, Pkcs11},
    mechanism::{
        rsa::{PkcsMgfType, PkcsPssParams},
        Mechanism, MechanismType,
    },
    object::{Attribute, AttributeType, KeyType, ObjectHandle},
    session::{Session, UserType},
    types::AuthPin,
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio_rustls::rustls::{
    self,
    pki_types::SubjectPublicKeyInfoDer,
    sign::{Signer, SigningKey},
    SignatureAlgorithm, SignatureScheme,
};

/// Errors occuring when handling a PKCS#11-based signing key.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Cryptoki error: {0}")]
    Cryptoki(#[from] cryptoki::error::Error),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Missing PIN")]
    MissingPin,
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Perform signature with private key.
#[derive(Clone, Debug)]
struct Pkcs11Signer {
    context: Pkcs11SigningContext,
    scheme: SignatureScheme,
}

impl Pkcs11Signer {
    /// Return the mechanism matching the scheme.
    fn mechanism(&self) -> Result<Mechanism, rustls::Error> {
        match self.scheme {
            SignatureScheme::RSA_PKCS1_SHA256 => Ok(Mechanism::Sha256RsaPkcs),
            SignatureScheme::RSA_PKCS1_SHA384 => Ok(Mechanism::Sha384RsaPkcs),
            SignatureScheme::RSA_PKCS1_SHA512 => Ok(Mechanism::Sha512RsaPkcs),
            SignatureScheme::RSA_PSS_SHA256 => {
                let params = PkcsPssParams {
                    hash_alg: MechanismType::SHA256,
                    mgf: PkcsMgfType::MGF1_SHA256,
                    s_len: 32.into(),
                };
                Ok(Mechanism::Sha256RsaPkcsPss(params))
            }
            SignatureScheme::RSA_PSS_SHA384 => {
                let params = PkcsPssParams {
                    hash_alg: MechanismType::SHA384,
                    mgf: PkcsMgfType::MGF1_SHA384,
                    s_len: 48.into(),
                };
                Ok(Mechanism::Sha384RsaPkcsPss(params))
            }
            SignatureScheme::RSA_PSS_SHA512 => {
                let params = PkcsPssParams {
                    hash_alg: MechanismType::SHA512,
                    mgf: PkcsMgfType::MGF1_SHA512,
                    s_len: 64.into(),
                };
                Ok(Mechanism::Sha512RsaPkcsPss(params))
            }
            _ => Err(rustls::Error::General(
                "Unsupported signature scheme".to_string(),
            )),
        }
    }
}

impl Signer for Pkcs11Signer {
    fn scheme(&self) -> SignatureScheme {
        self.scheme
    }

    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, rustls::Error> {
        let mechanism = self.mechanism()?;
        let session = self.context.session.lock().unwrap();
        let data = session
            .sign(&mechanism, self.context.key, message)
            .map_err(|e| rustls::Error::General(format!("Failed to sign: {e}")))?;
        Ok(data)
    }
}

/// Hold everything to sign.
#[derive(Clone, Debug)]
struct Pkcs11SigningContext {
    session: Arc<Mutex<Session>>,
    key: ObjectHandle,
}

/// Represent a private signing key, in a PKCS#11 token.
#[derive(Clone, Debug)]
pub(crate) struct Pkcs11SigningKey {
    #[allow(dead_code)]
    pkcs11: Pkcs11,
    context: Pkcs11SigningContext,
}

impl Pkcs11SigningKey {
    /// Create a new PKCS#11-based signing key.
    pub(crate) fn new(uri: &Pkcs11Uri) -> Result<Self, Error> {
        let pkcs11 = Pkcs11::new(uri.module_path())?;
        pkcs11.initialize(CInitializeArgs::OsThreads)?;
        let session = Self::open_session(&pkcs11, uri)?;
        let key_template = vec![
            Attribute::Label(uri.object().as_bytes().to_vec()),
            Attribute::Private(true),
            Attribute::Sign(true),
            Attribute::KeyType(KeyType::RSA),
        ];
        let keys = session.find_objects(&key_template)?;
        if keys.is_empty() {
            return Err(Error::NotFound("No such PKCS#11 key".to_string()));
        }
        Ok(Self {
            pkcs11,
            context: Pkcs11SigningContext {
                session: Arc::new(Mutex::new(session)),
                key: keys[0],
            },
        })
    }
    /// Open session.
    fn open_session(pkcs11: &Pkcs11, uri: &Pkcs11Uri) -> Result<Session, Error> {
        let slots = pkcs11.get_slots_with_initialized_token()?;
        if slots.is_empty() {
            return Err(Error::NotFound("No PKCS#11 token found".to_string()));
        }
        let pin = uri.pin().ok_or_else(|| Error::MissingPin)?;
        let pin = AuthPin::new(pin.into());
        let session = pkcs11.open_ro_session(slots[0])?;
        session.login(UserType::User, Some(&pin))?;
        Ok(session)
    }
    /// Return the list of supported signature schemes.
    #[allow(clippy::unused_self)]
    fn supported_schemes(&self) -> &[SignatureScheme] {
        &[
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
        ]
    }
}

impl SigningKey for Pkcs11SigningKey {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::RSA
    }

    fn choose_scheme(&self, offered: &[SignatureScheme]) -> Option<Box<dyn Signer>> {
        let supported = self.supported_schemes();
        for scheme in offered {
            if supported.contains(scheme) {
                return Some(Box::new(Pkcs11Signer {
                    context: self.context.clone(),
                    scheme: *scheme,
                }));
            }
        }
        None
    }

    fn public_key(&self) -> Option<SubjectPublicKeyInfoDer<'_>> {
        let session = self.context.session.lock().unwrap();
        session
            .get_attributes(self.context.key, &[AttributeType::PublicKeyInfo])
            .ok()
            .and_then(|attrs| attrs.into_iter().next())
            .and_then(|attr| {
                if let Attribute::PublicKeyInfo(value) = attr {
                    Some(value.into())
                } else {
                    None
                }
            })
    }
}
