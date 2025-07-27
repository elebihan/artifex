//
// Copyright (C) 2022-2024 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")?;
    let builder = if target_arch.as_str() == "wasm32" {
        tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .build_transport(false)
    } else {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        tonic_build::configure().file_descriptor_set_path(out_dir.join("artifex_descriptor.bin"))
    };
    builder.compile_protos(&["proto/artifex.proto"], &["proto"])?;
    Ok(())
}
