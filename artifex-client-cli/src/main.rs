//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_batch::{Batch, BatchRunner, MarkupKind, MarkupReportRenderer};
use artifex_rpc::artifex_client::ArtifexClient;
use clap::Parser;
use tonic::transport::Endpoint;

const BATCH_DEFAULT: &str = r#"
INSPECT
EXECUTE: date -u
UPGRADE
EXECUTE: uptime
"#;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(
        short,
        long,
        help = "URL of the server",
        default_value = "http://127.0.0.1:50051"
    )]
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let endpoint = Endpoint::from_shared(args.url)?;
    let client = ArtifexClient::connect(endpoint).await?;
    let batch = Batch::from_reader(BATCH_DEFAULT.as_bytes())?;
    let mut runner = BatchRunner::new(client);
    let report = runner.run(&batch).await?;
    let renderer = MarkupReportRenderer::new(MarkupKind::Yaml);
    let mut stdout = std::io::stdout().lock();
    renderer.render(&mut stdout, &report)?;
    Ok(())
}
