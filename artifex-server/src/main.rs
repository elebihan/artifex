//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use anyhow::{Context, Result};
use artifex_rpc::{artifex_server::ArtifexServer, FILE_DESCRIPTOR_SET};
use artifex_server::service::ArtifexService;
use clap::Parser;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_web;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help = "Address to use", default_value = "127.0.0.1")]
    address: String,

    #[arg(short, long, help = "Port to use", default_value_t = 50051)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let address = args
        .address
        .parse()
        .with_context(|| "failed to parse address")?;
    let address = SocketAddr::new(address, args.port);
    let artifex = ArtifexService::default();
    let server = ArtifexServer::new(artifex);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;

    let artifex = tonic_web::enable(server);

    Server::builder()
        .accept_http1(true)
        .add_service(artifex)
        .add_service(reflection)
        .serve(address)
        .await
        .with_context(|| "failed to start server")?;
    Ok(())
}
