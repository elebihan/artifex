//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::{fs::File, io::Write, path::PathBuf};

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
    #[arg(short, long, help = "Path to report file")]
    report: Option<PathBuf>,
    #[arg(help = "Path to batch file")]
    batch: Option<PathBuf>,
}

impl Cli {
    fn report(&self) -> Result<Box<dyn Write>, std::io::Error> {
        match &self.report {
            Some(path) => File::create(path).map(|f| Box::new(f) as Box<dyn Write>),
            None => Ok(Box::new(std::io::stdout().lock())),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let mut output = args.report()?;
    let endpoint = Endpoint::from_shared(args.url)?;
    let client = ArtifexClient::connect(endpoint).await?;
    let batch = match args.batch {
        Some(path) if path.as_os_str() == "-" => Batch::from_reader(std::io::stdin())?,
        Some(path) => Batch::from_file(path)?,
        None => Batch::from_reader(BATCH_DEFAULT.as_bytes())?,
    };
    let mut runner = BatchRunner::new(client);
    let report = runner.run(&batch).await?;
    let renderer = MarkupReportRenderer::new(MarkupKind::Yaml);
    renderer.render(&mut output, &report)?;
    Ok(())
}
