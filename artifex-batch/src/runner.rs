//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use crate::{
    batch::Batch,
    command::{Command, CommandOutput, CommandStatus},
    error::Error,
    report::{BatchReport, ReportEntry},
};

use artifex_rpc::{artifex_client::ArtifexClient, ExecuteRequest, InspectRequest, UpgradeRequest};
use futures_util::StreamExt;
use humantime::format_duration;
use std::{fmt::Write, time::Duration};
use uuid::Uuid;

/// Run commands via a client.
#[derive(Debug)]
pub(crate) struct CommandRunner {
    client: ArtifexClient<tonic::transport::Channel>,
}

impl CommandRunner {
    /// Run a command and return its output.
    pub(crate) async fn run(&mut self, command: &Command) -> Result<CommandStatus, Error> {
        let status = match command {
            Command::Execute(command) => {
                let response = self
                    .client
                    .execute(ExecuteRequest {
                        command: command.to_string(),
                    })
                    .await?;
                let reply = response.into_inner();
                CommandStatus::Success(Some(CommandOutput::String(reply.stdout)))
            }
            Command::Inspect => {
                let response = self.client.inspect(InspectRequest {}).await?;
                let reply = response.into_inner();
                let output = format!(
                    "kernel version: {}\nsystem uptime: {}",
                    reply.kernel_version,
                    format_duration(Duration::from_secs(reply.system_uptime))
                );
                CommandStatus::Success(Some(CommandOutput::String(output)))
            }
            Command::Upgrade => {
                let response = self.client.upgrade(UpgradeRequest {}).await?;
                let mut output = String::new();
                let mut stream = response.into_inner();
                while let Some(reply) = stream.next().await {
                    let progress = reply?;
                    writeln!(
                        output,
                        "Upgrade progress: {:?}, {}%",
                        progress.status(),
                        progress.position
                    )?;
                }
                CommandStatus::Success(Some(CommandOutput::String(output)))
            }
        };
        Ok(status)
    }
}

/// Allow to run batches of commands via a client.
#[derive(Debug)]
pub struct BatchRunner {
    inner: CommandRunner,
}

impl BatchRunner {
    /// Build a `BatchRunner` associated to a `ArtifexClient`
    pub fn new(client: ArtifexClient<tonic::transport::Channel>) -> Self {
        Self {
            inner: CommandRunner { client },
        }
    }

    /// Run a batch of commands
    pub async fn run(&mut self, batch: &Batch) -> Result<BatchReport, Error> {
        let title = format!("Report - {}", Uuid::new_v4());
        let mut report = BatchReport::new(&title);
        for command in &batch.commands {
            let status = self.inner.run(command).await?;
            report.push(ReportEntry {
                command: command.clone(),
                status,
            });
        }
        Ok(report)
    }
}
