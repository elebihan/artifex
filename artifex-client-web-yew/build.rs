//
// Copyright (C) 2023 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["../artifex-rpc/proto/artifex.proto"],
            &["../artifex-rpc/proto"],
        )?;
    tonic_build::compile_protos("../artifex-rpc/proto/artifex.proto")?;
    Ok(())
}
