//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use crate::config::Config;
use crate::error::{Error, Result};
use crate::machine::{MachineInfo, get_machine_info};
use rand::{self, Rng};
use random_progression::RandomProgression;
use std::ffi::OsStr;
use std::path::Path;

pub struct ProgramOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Default)]
pub struct Engine {
    config: Config,
}

impl Engine {
    /// Create a new `Engine` with the configuration `config`.
    #[must_use]
    pub fn with_config(config: Config) -> Self {
        Self { config }
    }

    pub fn inspect(&self) -> Result<MachineInfo> {
        get_machine_info()
    }

    pub fn execute<I, S>(&self, program: S, args: I) -> Result<ProgramOutput>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let program = program.as_ref();
        let program = Path::new(program).file_name().unwrap_or(program);
        if let Some(program) = self.config.allowed_programs().find(|p| *p == program) {
            let output = std::process::Command::new(program).args(args).output()?;
            Ok(ProgramOutput {
                code: output.status.code().unwrap_or(-1),
                stdout: std::str::from_utf8(&output.stdout)?.into(),
                stderr: std::str::from_utf8(&output.stderr)?.into(),
            })
        } else {
            Err(Error::Internal("Program not allowed".to_string()))
        }
    }

    pub fn upgrade<F>(&self, notify: F) -> Result<()>
    where
        F: Fn(u8),
    {
        let progression = RandomProgression::new();
        let mut rng = rand::rng();
        let delay: u16 = rng.random_range(500..2000);
        let duration = std::time::Duration::from_millis(u64::from(delay));
        for position in progression {
            std::thread::sleep(duration);
            notify(position);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_progressive_stuff() {
        let engine = Engine::default();
        let res = engine.upgrade(|position| {
            println!("Progression: {position}%");
        });
        assert!(res.is_ok());
    }
}
