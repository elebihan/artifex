//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_batch::{Batch, BatchRunner, MarkupKind, MarkupReportRenderer};
use artifex_rpc::artifex_client::ArtifexClient;

const BATCH_DEFAULT: &str = r#"
INSPECT
EXECUTE: date -u
UPGRADE
EXECUTE: uptime
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ArtifexClient::connect("http://127.0.0.1:50051").await?;
    let batch = Batch::from_reader(BATCH_DEFAULT.as_bytes())?;
    let mut runner = BatchRunner::new(client);
    let report = runner.run(&batch).await?;
    let renderer = MarkupReportRenderer::new(MarkupKind::Yaml);
    let mut stdout = std::io::stdout().lock();
    renderer.render(&mut stdout, &report)?;
    Ok(())
}
