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
    pki_types::{pem::PemObject, PrivateKeyDer, SubjectPublicKeyInfoDer},
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

/// Represent a private signing key.
#[derive(Clone, Debug)]
pub(super) struct ClientSigningKey {
    inner: RsaPrivateKey,
}

impl ClientSigningKey {
    /// Create a signing key from a PEM file.
    pub(super) fn with_pem_file<P: AsRef<Path>>(key_path: P) -> Result<Self> {
        let inner = PrivateKeyDer::from_pem_file(&key_path).with_context(|| {
            format!(
                "Failed to create private key from {}",
                key_path.as_ref().display()
            )
        })?;
        let inner = RsaPrivateKey::from_pkcs8_der(inner.secret_der()).with_context(|| {
            format!(
                "Failed to create RSA key from {}",
                key_path.as_ref().display()
            )
        })?;
        Ok(Self { inner })
    }

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
