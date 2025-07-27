//
// Copyright (C) 2022-2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use serde::Deserialize;

/// Hold the configuration of the engine.
#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct Config {
    pub allowed_programs: Vec<String>,
}

impl Config {
    /// Return an iterator over the allowed programs.
    pub fn allowed_programs(&self) -> impl Iterator<Item = &str> {
        self.allowed_programs
            .iter()
            .map(std::string::String::as_str)
    }
}
