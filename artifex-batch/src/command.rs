//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::{fmt::Display, str::FromStr};
use thiserror::Error;

/// Errors raised when handling a `Command`.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Empty string")]
    EmptyString,
    #[error("Missing argument")]
    MissingArgument,
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
}

/// Represent a command to be execute via a client.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Execute(String),
    Inspect,
    Upgrade,
}

impl FromStr for Command {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(Error::EmptyString);
        }
        match s.split_once(':') {
            Some(("EXECUTE", arg)) => Ok(Command::Execute(arg.trim().to_string())),
            Some((cmd, _)) => Err(Error::UnknownCommand(cmd.to_string())),
            None => match s {
                "EXECUTE" => Err(Error::MissingArgument),
                "INSPECT" => Ok(Command::Inspect),
                "UPGRADE" => Ok(Command::Upgrade),
                _ => Err(Error::UnknownCommand(s.to_string())),
            },
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Execute(command) => write!(f, "EXECUTE: {command}"),
            Command::Inspect => write!(f, "INSPECT"),
            Command::Upgrade => write!(f, "UPGRADE"),
        }
    }
}

/// Hold the output the execution of a command.
#[derive(Debug, PartialEq)]
pub enum CommandOutput {
    String(String),
    Uint32(u32),
}

impl Display for CommandOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandOutput::String(s) => write!(f, "{s}"),
            CommandOutput::Uint32(u) => write!(f, "{u}"),
        }
    }
}

/// Represent the status of the execution of a command.
#[derive(Debug, PartialEq)]
pub enum CommandStatus {
    Success(Option<CommandOutput>),
    Failure,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_execute() {
        let res = "EXECUTE: date -u".parse::<Command>();
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Command::Execute("date -u".to_string()));
    }
}
