//
// Copyright (C) 2022-2026 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! File upload session management.

use blake3::Hasher;
use hex;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tracing::debug;

/// Errors reported during an upload session.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Hash mismatch: expected {0}, computed {1}")]
    HashMismatch(String, String),
    #[error("Invalid file name: {0}")]
    InvalidFileName(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Size mismatch: expected {0}, received {1}")]
    SizeMismatch(u64, u64),
}

/// Hold information about a file upload session.
#[derive(Debug)]
pub struct UploadSession {
    file: File,
    expected_size: u64,
    received_size: u64,
    hasher: Hasher,
    temp_path: PathBuf,
    final_path: PathBuf,
}

impl UploadSession {
    /// Create a new `UploadSession`.
    pub async fn new(
        file_name: &str,
        expected_size: u64,
        upload_dir: &Path,
    ) -> Result<Self, Error> {
        let file_name = Path::new(file_name)
            .file_name()
            .ok_or(Error::InvalidFileName(file_name.to_string()))
            .map(|n| n.to_string_lossy())?;
        let final_path = upload_dir.join(file_name.as_ref());
        let temp_path = upload_dir.join(format!("{file_name}.part"));
        let file = File::create(&temp_path).await?;
        let hasher = Hasher::new();
        debug!("Creating upload session file {}", temp_path.display());
        Ok(Self {
            file,
            expected_size,
            hasher,
            received_size: 0,
            temp_path,
            final_path,
        })
    }
    /// Write a chunk of data from uploaded file.
    pub async fn write_chunk(&mut self, data: &[u8]) -> Result<(), Error> {
        let () = self.file.write_all(data).await?;
        self.hasher.update(data);
        self.received_size += data.len() as u64;
        Ok(())
    }
    /// Finalize upload session, comparing the computed hash to the expected value.
    pub async fn finalize(&mut self, expected_hash: &[u8]) -> Result<PathBuf, Error> {
        if self.expected_size != self.received_size {
            return Err(Error::SizeMismatch(self.expected_size, self.received_size));
        }
        let computed_hash = self.hasher.finalize();
        let computed_hash = computed_hash.as_slice();
        if computed_hash != expected_hash {
            return Err(Error::HashMismatch(
                hex::encode(expected_hash),
                hex::encode(computed_hash),
            ));
        }
        fs::rename(&self.temp_path, &self.final_path).await?;
        debug!(
            "Renamed {} to {}",
            self.temp_path.display(),
            self.final_path.display()
        );
        Ok(self.final_path.clone())
    }
}

impl Drop for UploadSession {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.temp_path);
    }
}
