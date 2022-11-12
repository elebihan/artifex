//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("artifex_descriptor.bin"))
        .compile(&["proto/artifex.proto"], &["proto"])
        .unwrap();
    tonic_build::compile_protos("proto/artifex.proto")?;
    Ok(())
}
