//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use anyhow::{Context, Result};
use artifex_batch::{Batch, BatchRunner, MarkupKind, MarkupReportRenderer};
use artifex_client_cli::{
    client::ClientBuilder, config::Config, password::PasswordProvider, tls::Config as TlsConfig,
};
use clap::{Parser, ValueEnum};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

const BATCH_DEFAULT: &str = r#"
INSPECT
EXECUTE: date -u
UPGRADE
EXECUTE: uptime
"#;

/// Format of the report
#[derive(Clone, Debug, ValueEnum)]
enum ReportFormat {
    Xml,
    Yaml,
}

impl From<ReportFormat> for MarkupKind {
    fn from(val: ReportFormat) -> Self {
        match val {
            ReportFormat::Xml => MarkupKind::Xml,
            ReportFormat::Yaml => MarkupKind::Yaml,
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(
        short = 'C',
        long,
        help = "Path to configuration file",
        value_name = "FILE"
    )]
    config: Option<PathBuf>,
    #[arg(short = 'F', long, value_enum)]
    format: Option<ReportFormat>,
    #[arg(
        short = 'U',
        long,
        help = "URL of the server",
        default_value = Config::DEFAULT_URL
    )]
    url: Option<String>,
    #[arg(
        short = 'S',
        long = "password",
        help = "Password provider",
        value_name = "PROVIDER"
    )]
    password_provider: Option<PasswordProvider>,
    #[arg(short = 'R', long, help = "Path to report file", value_name = "FILE")]
    report: Option<PathBuf>,
    #[arg(
        short = 'r',
        long,
        help = "Root certificate authority file",
        value_name = "FILE"
    )]
    pub root_cert: Option<PathBuf>,
    #[arg(
        short = 'c',
        long,
        help = "Client certificate file",
        value_name = "FILE"
    )]
    pub client_cert: Option<PathBuf>,
    #[arg(
        short = 'k',
        long,
        help = "Client private key file",
        value_name = "FILE"
    )]
    pub client_key: Option<PathBuf>,
    #[arg(
        short = 'n',
        long,
        help = "Server alternative name",
        value_name = "NAME"
    )]
    pub server_alt_name: Option<String>,
    #[arg(help = "Path to batch file")]
    batch: Option<PathBuf>,
}

impl Cli {
    fn batch(&self) -> Result<Box<dyn Read>, std::io::Error> {
        match &self.batch {
            Some(path) if path.as_os_str() == "-" => Ok(Box::new(std::io::stdin())),
            Some(path) => File::open(path).map(|f| Box::new(f) as Box<dyn Read>),
            None => Ok(Box::new(BATCH_DEFAULT.as_bytes())),
        }
    }
    fn report(&self) -> Result<Box<dyn Write>, std::io::Error> {
        match &self.report {
            Some(path) => File::create(path).map(|f| Box::new(f) as Box<dyn Write>),
            None => Ok(Box::new(std::io::stdout().lock())),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let config = if let Some(path) = &args.config {
        Config::with_path(path)?
    } else {
        Config::default()
    };
    let input = args.batch().with_context(|| "failed to open input")?;
    let mut output = args.report().with_context(|| "failed to create report")?;
    let url = args.url.unwrap_or(config.url);
    let password_provider = args.password_provider.unwrap_or(config.password_provider);
    let builder = if let Some(("https", _)) = url.split_once("://") {
        let password = password_provider
            .provide()
            .map(|s| if s.trim().is_empty() { None } else { Some(s) })
            .with_context(|| "Failed to get password")?;
        let tls = TlsConfig {
            root_cert: args.root_cert.unwrap_or(config.tls.root_cert),
            client_cert: args.client_cert.unwrap_or(config.tls.client_cert),
            client_key: args.client_key.unwrap_or(config.tls.client_key),
            client_password: password,
            server_alt_name: args.server_alt_name.or(config.tls.server_alt_name),
        };
        ClientBuilder::with_tls_config(tls)
    } else {
        ClientBuilder::default()
    };
    let mut client = builder
        .connect(&url)
        .await
        .with_context(|| "failed to connect to server")?;
    let mut runner = BatchRunner::new(&mut client);
    let batch = Batch::from_reader(input).with_context(|| "failed to open batch")?;
    let report = runner
        .run(&batch)
        .await
        .with_context(|| "failed to run batch")?;
    let format = args.format.map(ReportFormat::into).unwrap_or(config.format);
    let renderer = MarkupReportRenderer::new(format);
    renderer
        .render(&mut output, &report)
        .with_context(|| "failed to render report")?;
    Ok(())
}
