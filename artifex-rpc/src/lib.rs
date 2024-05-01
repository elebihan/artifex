//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

tonic::include_proto!("artifex");

#[cfg(not(target_arch = "wasm32"))]
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("artifex_descriptor");
