//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! Password management.

use rpassword;
use serde::Deserialize;
use std::path::PathBuf;
#[cfg(unix)]
use std::{io::Read, os::unix::prelude::FromRawFd};
use thiserror::Error;

/// Errors reported when handling password.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid provider: {0}")]
    InvalidProvider(String),
    #[error("Parsing error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}

fn trim_newline(text: &mut String) -> &mut String {
    let new_len = text
        .char_indices()
        .rev()
        .find(|(_, c)| !matches!(c, '\n' | '\r'))
        .map_or(0, |(i, _)| i + 1);
    if new_len != text.len() {
        text.truncate(new_len);
    }
    text
}

/// Read password from various sources.
///
/// A password provider can be created from its string representation, formatted
/// as `<SCHEME>:<VALUE>`, where `<SCHEME>` can be:
///
/// - `env`: read password from environment variable, which name is passed in
///          `<VALUE>`.
/// - `fd`: read password from file description specified by `<VALUE>`.
/// - `file`: read password from file specified by `<VALUE>`.
///
/// Any other scheme results in prompting the user for the password.
///
/// Examples:
///
/// - `env:FOO` tells to use the value of environment variable "FOO" as password.
/// - `fd:3` tells to read password from file descriptor 3.
/// - `file:/dev/null` tells to read password from `/dev/null`.
///
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub enum PasswordProvider {
    /// Read password from an environment variable.
    Env(String),
    #[cfg(unix)]
    /// Read password from a file descriptor.
    Fd(u8),
    /// Read password from a file.
    File(PathBuf),
    #[default]
    /// Read password interactively, from standard input, hiding the value.
    Prompt,
}

impl std::str::FromStr for PasswordProvider {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((name, value)) = s.split_once(':') {
            let provider = match name {
                #[cfg(unix)]
                "fd" => {
                    let fd = value.parse::<u8>()?;
                    PasswordProvider::Fd(fd)
                }
                "file" => {
                    let path = PathBuf::from(value);
                    PasswordProvider::File(path)
                }
                "env" => PasswordProvider::Env(value.to_string()),
                _ => PasswordProvider::Prompt,
            };
            Ok(provider)
        } else {
            Err(Error::InvalidProvider(s.to_string()))
        }
    }
}

impl PasswordProvider {
    /// Provide password value, reading it its source.
    pub fn provide(&self) -> Result<String, Error> {
        let mut password = match self {
            PasswordProvider::Env(var) => std::env::var(var)?,
            #[cfg(unix)]
            PasswordProvider::Fd(fd) => {
                let mut file = unsafe { std::fs::File::from_raw_fd(*fd as i32) };
                let mut password = String::new();
                file.read_to_string(&mut password)?;
                password
            }
            PasswordProvider::File(path) => std::fs::read_to_string(path)?,
            PasswordProvider::Prompt => rpassword::prompt_password("Please enter password: ")?,
        };
        trim_newline(&mut password);
        Ok(password)
    }
}
