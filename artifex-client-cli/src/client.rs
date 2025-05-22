//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Client management.

use anyhow::{Context, Result};
use artifex_rpc::artifex_client::ArtifexClient;
use hyper_rustls::FixedServerNameResolver;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::connect::HttpConnector;
use tokio_rustls::rustls::pki_types::ServerName;
use tonic::transport::{Channel, Endpoint};
use tower::ServiceBuilder;

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
        let endpoint = Endpoint::from_shared(url.to_string()).with_context(|| "Invalid URI")?;
        let tls = if let Some(tls) = &self.tls {
            let tls = create_client_config(tls)
                .with_context(|| "failed to create TLS client configuration")?;
            Some(tls)
        } else {
            None
        };
        let conn = if let Some(tls) = tls {
            let mut http = HttpConnector::new();
            http.enforce_http(false);
            let url = endpoint.clone().uri().to_owned();
            let name = self
                .tls
                .and_then(|tls| tls.server_alt_name.map(ServerName::try_from))
                .transpose()?;
            let connector = ServiceBuilder::new()
                .layer_fn(move |s| {
                    if let Some(name) = name.clone() {
                        let resolver = FixedServerNameResolver::new(name);
                        HttpsConnectorBuilder::new()
                            .with_tls_config(tls.clone())
                            .https_only()
                            .with_server_name_resolver(resolver)
                            .enable_http2()
                            .wrap_connector(s)
                    } else {
                        HttpsConnectorBuilder::new()
                            .with_tls_config(tls.clone())
                            .https_only()
                            .enable_http2()
                            .wrap_connector(s)
                    }
                })
                .map_request(move |_| url.clone())
                .service(http);
            Channel::connect(connector, endpoint)
                .await
                .with_context(|| "failed to connect to server")?
        } else {
            endpoint
                .connect()
                .await
                .with_context(|| "failed to connect to server")?
        };
        let client = ArtifexClient::new(conn);
        Ok(client)
    }
}
