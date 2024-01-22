//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

pub mod rpc {
    tonic::include_proto!("artifex");

    use tonic_web_wasm_client::Client;

    /// Create a client for gRPC server.
    pub fn create_client(url: &str) -> self::artifex_client::ArtifexClient<Client> {
        let client = Client::new(url.to_string());
        self::artifex_client::ArtifexClient::new(client)
    }
}

pub mod components;
pub mod contexts;
