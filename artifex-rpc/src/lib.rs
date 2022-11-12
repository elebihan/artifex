//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

tonic::include_proto!("artifex");

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("artifex_descriptor");
