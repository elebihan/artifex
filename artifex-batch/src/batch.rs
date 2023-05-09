//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use super::command::Command;
use super::error::Error;
use std::io::BufRead;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
    str::FromStr,
};

/// Represent a batch of commands.
#[derive(Debug, PartialEq)]
pub struct Batch {
    pub(crate) commands: Vec<Command>,
}

impl FromStr for Batch {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let commands: Result<Vec<Command>, Error> = s
            .split(';')
            .map(|s| Command::from_str(s).map_err(Error::Syntax))
            .collect();
        Ok(Batch {
            commands: commands?,
        })
    }
}

impl Batch {
    /// Build a `Batch` from the contents of a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    pub fn from_reader<R: Read>(reader: R) -> Result<Self, Error> {
        let reader = BufReader::new(reader);
        let commands: Result<Vec<Command>, Error> = reader
            .lines()
            .filter_map(Result::ok)
            .filter(|l| !l.starts_with('#'))
            .filter(|l| !l.is_empty())
            .map(|l| Command::from_str(&l).map_err(Error::Syntax))
            .collect();
        Ok(Batch {
            commands: commands?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_string() {
        let res = "EXECUTE: date -u; UPGRADE".parse::<Batch>();
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            Batch {
                commands: vec![Command::Execute("date -u".to_string()), Command::Upgrade]
            }
        );
    }

    const BATCH_VALID: &str = r##"
INSPECT
# Comment
EXECUTE: date -u
UPGRADE
"##;

    #[test]
    fn parse_valid_text() {
        let res = Batch::from_reader(BATCH_VALID.as_bytes());
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            Batch {
                commands: vec![
                    Command::Inspect,
                    Command::Execute("date -u".to_string()),
                    Command::Upgrade
                ]
            }
        );
    }
}
