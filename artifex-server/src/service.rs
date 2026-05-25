//
// Copyright (C) 2022-2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_engine::{Config, Engine};
use artifex_rpc::{
    Digest, ExecuteReply, ExecuteRequest, InspectReply, InspectRequest, TransferEpilogue,
    UpgradeReply, UpgradeRequest, UploadRequest, UploadResponse, artifex_server::Artifex,
    upgrade_reply, upload_request::Payload,
};

use futures::Stream;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::{pin::Pin, sync::Arc};
use tokio::sync::mpsc;
use tokio::task;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, warn};

use crate::upload_session::UploadSession;

#[derive(Default)]
pub struct ArtifexService {
    engine: Arc<Mutex<Engine>>,
}

impl ArtifexService {
    /// Create a new service using an engine configuration.
    #[must_use]
    pub fn with_engine_config(config: Config) -> Self {
        Self {
            engine: Arc::new(Mutex::new(Engine::with_config(config))),
        }
    }
}

#[tonic::async_trait]
impl Artifex for ArtifexService {
    type UpgradeStream = Pin<Box<dyn Stream<Item = Result<UpgradeReply, Status>> + Send>>;

    async fn inspect(
        &self,
        _request: Request<InspectRequest>,
    ) -> Result<Response<InspectReply>, Status> {
        let engine = self.engine.lock().expect("Engine lock not poisoned");
        let info = engine
            .inspect()
            .map_err(|e| tonic::Status::internal(format!("Failed to inspect: {e}")))?;
        let response = InspectReply {
            kernel_version: info.kernel_version,
            system_uptime: info.system_uptime.as_secs(),
        };
        Ok(Response::new(response))
    }

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteReply>, Status> {
        let execute_req = request.into_inner();
        let mut args = execute_req.command.split_whitespace();
        let engine = self.engine.lock().expect("Engine lock not poisoned");
        if let Some(program) = args.next() {
            engine
                .execute(program, args)
                .map_err(|e| Status::internal(e.to_string()))
                .map(|output| {
                    let response = ExecuteReply {
                        code: output.code,
                        stdout: output.stdout,
                        stderr: output.stderr,
                    };
                    Response::new(response)
                })
        } else {
            Err(Status::invalid_argument("Missing program name"))
        }
    }

    async fn upgrade(
        &self,
        _request: Request<UpgradeRequest>,
    ) -> Result<Response<Self::UpgradeStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        let engine = self.engine.clone();
        let tx_clone = tx.clone();
        task::spawn_blocking(move || {
            let engine = engine.lock().expect("Engine lock not poisoned");
            let res = engine.upgrade(move |position| {
                let reply = UpgradeReply {
                    status: upgrade_reply::Status::Running as i32,
                    position: i32::from(position),
                };
                if tx_clone
                    .blocking_send(Result::<_, Status>::Ok(reply))
                    .is_err()
                {}
            });
            let status = if res.is_ok() {
                upgrade_reply::Status::Success as i32
            } else {
                upgrade_reply::Status::Failure as i32
            };
            let reply = UpgradeReply {
                status,
                position: 100,
            };
            let _ = tx.blocking_send(Result::<_, Status>::Ok(reply));
        });

        let ostream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(ostream) as Self::UpgradeStream))
    }
    /// Upload a file on a machine.
    async fn upload(
        &self,
        request: Request<Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
        let mut session: Option<UploadSession> = None;
        let mut file_path = None;
        let mut istream = request.into_inner();
        while let Some(request) = istream.message().await? {
            match request.payload {
                Some(Payload::Prologue(prologue)) => {
                    debug!("Received transfer prologue {prologue:?}");
                    session = UploadSession::new(
                        &prologue.file_name,
                        prologue.file_size,
                        Path::new("/tmp"),
                    )
                    .await
                    .map(Some)
                    .map_err(|e| Status::internal(format!("Failed to upload session: {e}")))?;
                }
                Some(Payload::Chunk(chunk)) => {
                    if let Some(session) = session.as_mut() {
                        session
                            .write_chunk(&chunk.data)
                            .await
                            .map_err(|e| Status::internal(format!("Failed to write chunk: {e}")))?;
                    } else {
                        return Err(Status::internal("No upload session available"));
                    }
                }
                Some(Payload::Epilogue(TransferEpilogue {
                    digest: Some(digest),
                })) => {
                    debug!("Received transfer epilogue with digest {digest:?}");
                    if let Some(session) = session.as_mut() {
                        let Digest {
                            kind: _,
                            data: expected_hash,
                        } = digest;
                        file_path = session
                            .finalize(expected_hash.as_slice())
                            .await
                            .map(Some)
                            .map_err(|e| Status::internal(e.to_string()))?;
                    } else {
                        return Err(Status::internal("No upload session available"));
                    }
                    break;
                }
                Some(Payload::Epilogue(TransferEpilogue { digest: _ })) => {
                    return Err(Status::internal("No digest received"));
                }
                Some(Payload::Error(error)) => {
                    error!("client-side error: {}", error.reason);
                    return Err(Status::internal(error.reason));
                }
                None => {
                    warn!("No payload received");
                }
            }
        }
        if let Some(file_path) = file_path {
            let file_size = fs::metadata(&file_path)
                .map_err(|e| Status::internal(format!("Failed to get uploaded file size: {e}")))
                .map(|m| m.len())?;
            let file_path = file_path.to_string_lossy().to_string();
            debug!("Uploaded as {file_path} ({file_size} bytes)");
            Ok(Response::new(UploadResponse {
                file_path,
                file_size,
            }))
        } else {
            Err(Status::internal("No file created"))
        }
    }
}
