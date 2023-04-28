//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_rpc::artifex_client::ArtifexClient;
use artifex_rpc::{ExecuteRequest, InspectRequest, UpgradeRequest};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ArtifexClient::connect("http://127.0.0.1:50051")
        .await
        .unwrap();

    let response = client.inspect(InspectRequest {}).await?;
    let reply = response.into_inner();
    println!("Kernel version: {}", reply.kernel_version);

    let response = client
        .execute(ExecuteRequest {
            command: "date -u".to_string(),
        })
        .await?;
    let reply = response.into_inner();
    println!("Date: {}", reply.stdout.trim());

    let response = client.upgrade(UpgradeRequest {}).await?;
    let mut stream = response.into_inner();
    while let Some(reply) = stream.next().await {
        let progress = reply?;
        println!(
            "Upgrade progress: {:?}, {}%",
            progress.status(),
            progress.position
        );
    }

    Ok(())
}
