//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use crate::error::Result;
use crate::machine::{get_machine_info, MachineInfo};
use rand::{thread_rng, Rng};
use random_progression::RandomProgression;
use std::ffi::OsStr;

pub struct ProgramOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Default)]
pub struct Engine;

impl Engine {
    pub fn inspect(&self) -> Result<MachineInfo> {
        get_machine_info()
    }

    pub fn execute<I, S>(&self, program: S, args: I) -> Result<ProgramOutput>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = std::process::Command::new(program).args(args).output()?;
        Ok(ProgramOutput {
            code: output.status.code().unwrap_or(-1),
            stdout: std::str::from_utf8(&output.stdout)?.into(),
            stderr: std::str::from_utf8(&output.stderr)?.into(),
        })
    }

    pub fn upgrade<F>(&self, notify: F) -> Result<()>
    where
        F: Fn(u8),
    {
        let mut progression = RandomProgression::new();
        let mut rng = thread_rng();
        let delay: u16 = rng.gen_range(500..2000);
        let duration = std::time::Duration::from_millis(delay as u64);
        while let Some(position) = progression.next() {
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
        let engine = Engine {};
        let res = engine.upgrade(|position| {
            println!("Progression: {}%", position);
        });
        assert!(res.is_ok());
    }
}
