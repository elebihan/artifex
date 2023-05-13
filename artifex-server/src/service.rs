//
// Copyright (C) 2022-2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_engine::Engine;
use artifex_rpc::{
    artifex_server::Artifex, upgrade_reply, ExecuteReply, ExecuteRequest, InspectReply,
    InspectRequest, UpgradeReply, UpgradeRequest,
};

use futures::Stream;
use std::sync::Mutex;
use std::{pin::Pin, sync::Arc};
use tokio::sync::mpsc;
use tokio::task;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct ArtifexService {
    engine: Arc<Mutex<Engine>>,
}

#[tonic::async_trait]
impl Artifex for ArtifexService {
    type UpgradeStream = Pin<Box<dyn Stream<Item = Result<UpgradeReply, Status>> + Send>>;

    async fn inspect(
        &self,
        _request: Request<InspectRequest>,
    ) -> Result<Response<InspectReply>, Status> {
        let engine = self.engine.lock().unwrap();
        let info = engine.inspect().unwrap();
        let response = InspectReply {
            kernel_version: info.kernel_version,
        };
        Ok(Response::new(response))
    }

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteReply>, Status> {
        let execute_req = request.into_inner();
        let mut args = execute_req.command.split_whitespace();
        let engine = self.engine.lock().unwrap();
        let program = args.next().unwrap();
        let output = engine.execute(program, args).unwrap();
        let response = ExecuteReply {
            code: output.code,
            stdout: output.stdout,
            stderr: output.stderr,
        };
        Ok(Response::new(response))
    }

    async fn upgrade(
        &self,
        _request: Request<UpgradeRequest>,
    ) -> Result<Response<Self::UpgradeStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        let engine = self.engine.clone();
        let tx_clone = tx.clone();
        task::spawn_blocking(move || {
            let engine = engine.lock().unwrap();
            let res = engine.upgrade(move |position| {
                let reply = UpgradeReply {
                    status: upgrade_reply::Status::Running as i32,
                    position: position as i32,
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
}
