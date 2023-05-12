//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::net::SocketAddr;

use artifex_rpc::{artifex_server::ArtifexServer, FILE_DESCRIPTOR_SET};
use artifex_server::service::ArtifexService;
use clap::Parser;
use http::Method;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help = "Address to use", default_value = "127.0.0.1")]
    address: String,

    #[arg(short, long, help = "Port to use", default_value_t = 50051)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let address = SocketAddr::new(args.address.parse()?, args.port);
    let artifex = ArtifexService::default();
    let server = ArtifexServer::new(artifex);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;

    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    Server::builder()
        .accept_http1(true)
        .layer(cors)
        .layer(GrpcWebLayer::new())
        .add_service(server)
        .add_service(reflection)
        .serve(address)
        .await?;
    Ok(())
}
