//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use thiserror::Error;

/// Errors raised when processing a batch.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Formatting error: {0}")]
    Fmt(#[from] std::fmt::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RPC error: {0}")]
    Rpc(#[from] tonic::Status),
    #[error("Syntax error: {0}")]
    Syntax(#[from] super::command::Error),
}
