//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use anyhow::{Context, Result};
use artifex_rpc::{artifex_server::ArtifexServer, FILE_DESCRIPTOR_SET};
use artifex_server::{config::Config, service::ArtifexService};
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tonic::transport::Server;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, help = "Address to use")]
    address: Option<String>,

    #[arg(short, long, help = "Port to use")]
    port: Option<u16>,

    #[arg(short = 'C', long, help = "Path to configuration file")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let config = if let Some(path) = args.config {
        Config::with_path(path)?
    } else {
        Config::default()
    };

    let address = args
        .address
        .unwrap_or(config.address)
        .parse()
        .with_context(|| "failed to parse address")?;
    let port = args.port.unwrap_or(config.port);
    let address = SocketAddr::new(address, port);

    let artifex = ArtifexService::with_engine_config(config.engine);
    let server = ArtifexServer::new(artifex);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;

    let artifex = tonic_web::enable(server);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .with_context(|| "Failed to init tracing")?;

    Server::builder()
        .accept_http1(true)
        .add_service(artifex)
        .add_service(reflection)
        .serve(address)
        .await
        .with_context(|| "failed to start server")?;
    Ok(())
}
