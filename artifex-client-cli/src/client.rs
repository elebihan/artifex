//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Client management.

use anyhow::{Context, Result};
use artifex_rpc::artifex_client::ArtifexClient;
use tonic::transport::Channel;

use crate::tls::{create_client_config, Config as TlsConfig};

/// Build a client, with options.
#[derive(Debug, Default)]
pub struct ClientBuilder {
    tls: Option<TlsConfig>,
}

impl ClientBuilder {
    /// Create a client builder, with a TLS configuration.
    pub fn with_tls_config(tls: TlsConfig) -> Self {
        Self { tls: Some(tls) }
    }
    /// Create a client for server at `url`.
    pub async fn connect(self, url: &str) -> Result<ArtifexClient<Channel>> {
        let tls = if let Some(tls) = &self.tls {
            let tls = create_client_config(tls)
                .with_context(|| "failed to create TLS client configuration")?;
            Some(tls)
        } else {
            None
        };
        let channel =
            Channel::from_shared(url.to_string()).with_context(|| "failed to create channel")?;
        let channel = if let Some(tls) = tls {
            channel
                .tls_config(tls)
                .with_context(|| "failed to set TLS client configuration")?
        } else {
            channel
        };
        let conn = channel
            .connect()
            .await
            .with_context(|| "failed to connect to server")?;
        let client = ArtifexClient::new(conn);
        Ok(client)
    }
}
