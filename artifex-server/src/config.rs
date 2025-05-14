//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use crate::error::Error;
use crate::tls::Config as TlsConfig;
use artifex_engine::Config as EngineConfig;
use serde::Deserialize;
use std::path::Path;

/// Hold the configuration of the server.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    /// Address to use.
    pub address: String,
    /// Port to use.
    pub port: u16,
    /// Configuration of the engine.
    pub engine: EngineConfig,
    /// Configuration of TLS.
    pub tls: TlsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: Self::DEFAULT_ADDRESS.to_string(),
            port: Self::DEFAULT_PORT,
            engine: EngineConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl Config {
    /// Default server address.
    pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
    /// Default server port.
    pub const DEFAULT_PORT: u16 = 50051;
    /// Create a new configuration from file at `path`.
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)?;
        let config = toml::de::from_str(&text)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_VALID: &str = r#"
address = "127.0.0.1"
port = 50051
[engine]
allowed_programs = ["date", "uname"]
[tls]
root_cert = "/etc/artifex/root.crt.pem"
server_cert = "/etc/artifex/server.crt.pem"
server_key = "/etc/artifex/server.key.pem"
"#;

    #[test]
    fn parse_config_valid() {
        let result = toml::de::from_str::<Config>(CONFIG_VALID);
        assert!(result.is_ok());
        let reference = Config {
            engine: EngineConfig {
                allowed_programs: vec!["date".to_string(), "uname".to_string()],
            },
            ..Default::default()
        };
        assert_eq!(result.unwrap(), reference);
    }
}
