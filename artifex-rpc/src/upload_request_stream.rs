//
// Copyright (C) 2022-2026 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

//! File upload stream management.

use blake3::Hasher;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio_stream::Stream;

use crate::{
    Digest, DigestKind, TransferChunk, TransferEpilogue, TransferError, TransferPrologue,
    UploadRequest, upload_request::Payload,
};

/// State of the file upload.
#[derive(Debug)]
enum UploadState {
    Begin,
    Send,
    End,
    Error,
}

/// Generate a stream of `UploadRequest`.
#[derive(Debug)]
pub struct UploadRequestStream {
    /// File to upload.
    file: File,
    /// Name of the file to upload.
    file_name: String,
    /// Size of the file to upload.
    file_size: u64,
    /// Hasher for the file to upload.
    hasher: Hasher,
    /// State of the file upload.
    state: UploadState,
}

impl UploadRequestStream {
    const BUFFER_SIZE: usize = 128 * 1024;
    /// Create a new `UploadRequestStream` for file at `path`.
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or(std::io::Error::other("Invalid file name"))?;
        let file = File::open(path)?;
        let file_size = file.metadata().map(|m| m.len())?;
        let hasher = Hasher::new();
        let state = UploadState::Begin;
        Ok(Self {
            file,
            file_name,
            file_size,
            hasher,
            state,
        })
    }
    /// Create a [`tokio_stream::Stream`] of `UploadRequest` for file at `path`.
    pub fn create<P: AsRef<Path>>(
        path: P,
    ) -> Result<impl Stream<Item = UploadRequest>, std::io::Error> {
        let it = UploadRequestStream::new(path)?;
        Ok(tokio_stream::iter(it))
    }
}

impl Iterator for UploadRequestStream {
    type Item = UploadRequest;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            UploadState::Begin => {
                self.state = UploadState::Send;
                let payload = Some(Payload::Prologue(TransferPrologue {
                    file_name: self.file_name.clone(),
                    file_size: self.file_size,
                }));
                Some(UploadRequest { payload })
            }
            UploadState::Send => {
                let mut data = vec![0; Self::BUFFER_SIZE];
                let payload = match self.file.read(&mut data) {
                    Ok(count) => {
                        if count == 0 {
                            let hash = self.hasher.finalize();
                            let digest = Digest {
                                kind: DigestKind::Blake3.into(),
                                data: hash.as_slice().to_vec(),
                            };
                            self.state = UploadState::End;
                            Payload::Epilogue(TransferEpilogue {
                                digest: Some(digest),
                            })
                        } else {
                            data.truncate(count);
                            self.hasher.update(&data);
                            Payload::Chunk(TransferChunk { data: data.clone() })
                        }
                    }
                    Err(e) => {
                        self.state = UploadState::Error;
                        Payload::Error(TransferError {
                            reason: e.to_string(),
                        })
                    }
                };
                Some(UploadRequest {
                    payload: Some(payload),
                })
            }
            UploadState::End | UploadState::Error => None,
        }
    }
}
