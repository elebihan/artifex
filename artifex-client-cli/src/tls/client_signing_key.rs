//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use anyhow::{Context, Result};
use rsa::{
    pkcs1::EncodeRsaPublicKey,
    pkcs1v15,
    pkcs8::DecodePrivateKey,
    pss,
    sha2::{Sha256, Sha384, Sha512},
    signature::{RandomizedSigner, SignatureEncoding},
    traits::PublicKeyParts,
    RsaPrivateKey,
};
use std::path::Path;
use tokio_rustls::rustls::{
    self,
    pki_types::SubjectPublicKeyInfoDer,
    sign::{Signer, SigningKey},
    SignatureAlgorithm, SignatureScheme,
};

/// Perform signature with private key.
#[derive(Clone, Debug)]
struct ClientSigner {
    key: RsaPrivateKey,
    scheme: SignatureScheme,
}

impl Signer for ClientSigner {
    fn scheme(&self) -> SignatureScheme {
        self.scheme
    }

    fn sign(&self, message: &[u8]) -> std::result::Result<Vec<u8>, rustls::Error> {
        let mut rng = rand::thread_rng();
        match self.scheme {
            SignatureScheme::RSA_PKCS1_SHA256 => {
                let key = pkcs1v15::SigningKey::<Sha256>::new(self.key.clone());
                Ok(key.sign_with_rng(&mut rng, message).to_vec())
            }
            SignatureScheme::RSA_PKCS1_SHA384 => {
                let key = pkcs1v15::SigningKey::<Sha384>::new(self.key.clone());
                Ok(key.sign_with_rng(&mut rng, message).to_vec())
            }
            SignatureScheme::RSA_PKCS1_SHA512 => {
                let key = pkcs1v15::SigningKey::<Sha512>::new(self.key.clone());
                Ok(key.sign_with_rng(&mut rng, message).to_vec())
            }
            SignatureScheme::RSA_PSS_SHA256 => {
                let key = pss::BlindedSigningKey::<Sha256>::new(self.key.clone());
                Ok(key.sign_with_rng(&mut rng, message).to_vec())
            }
            SignatureScheme::RSA_PSS_SHA384 => {
                let key = pss::BlindedSigningKey::<Sha384>::new(self.key.clone());
                Ok(key.sign_with_rng(&mut rng, message).to_vec())
            }
            SignatureScheme::RSA_PSS_SHA512 => {
                let key = pss::BlindedSigningKey::<Sha512>::new(self.key.clone());
                Ok(key.sign_with_rng(&mut rng, message).to_vec())
            }
            _ => Err(rustls::Error::General(
                "Unsupported signature scheme".to_string(),
            )),
        }
    }
}

// A builder for configuring a client signing key.
#[derive(Debug)]
pub(super) struct ClientSigningKeyBuilder {
    key_pem: String,
    password: Option<String>,
}

impl ClientSigningKeyBuilder {
    /// Create a new client signing key builder.
    pub(super) fn with_pem_file<P: AsRef<Path>>(key_path: P) -> Result<Self> {
        let key_pem = std::fs::read_to_string(&key_path).with_context(|| {
            format!(
                "Failed to read private key from {}",
                key_path.as_ref().display()
            )
        })?;
        Ok(Self {
            key_pem,
            password: None,
        })
    }
    /// Set password for key decryption.
    pub(super) fn password(&mut self, password: &str) -> &Self {
        self.password = Some(password.to_string());
        self
    }
    /// Build a client signing key.
    pub(super) fn build(self) -> Result<ClientSigningKey> {
        let inner = if let Some(password) = &self.password {
            RsaPrivateKey::from_pkcs8_encrypted_pem(&self.key_pem, password)
        } else {
            RsaPrivateKey::from_pkcs8_pem(&self.key_pem)
        };
        let inner = inner.with_context(|| "Failed to create RSA key")?;
        Ok(ClientSigningKey { inner })
    }
}

/// Represent a private signing key.
#[derive(Clone, Debug)]
pub(super) struct ClientSigningKey {
    inner: RsaPrivateKey,
}

impl ClientSigningKey {
    /// Return the list of supported signature schemes.
    fn supported_schemes(&self) -> &[SignatureScheme] {
        match self.inner.to_public_key().size() {
            256 => &[
                SignatureScheme::RSA_PKCS1_SHA256,
                SignatureScheme::RSA_PSS_SHA256,
            ],
            384 => &[
                SignatureScheme::RSA_PKCS1_SHA384,
                SignatureScheme::RSA_PSS_SHA384,
            ],
            512 => &[
                SignatureScheme::RSA_PKCS1_SHA512,
                SignatureScheme::RSA_PSS_SHA512,
            ],
            _ => unreachable!(),
        }
    }
}

impl SigningKey for ClientSigningKey {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::RSA
    }

    fn choose_scheme(&self, offered: &[SignatureScheme]) -> Option<Box<dyn Signer>> {
        let supported = self.supported_schemes();
        for scheme in offered {
            if supported.contains(scheme) {
                return Some(Box::new(ClientSigner {
                    key: self.inner.clone(),
                    scheme: *scheme,
                }));
            }
        }
        None
    }

    fn public_key(&self) -> Option<SubjectPublicKeyInfoDer<'_>> {
        self.inner
            .to_public_key()
            .to_pkcs1_der()
            .ok()
            .map(|d| d.to_vec().into())
    }
}
